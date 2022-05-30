//
//  TunnelManager.swift
//  MullvadVPN
//
//  Created by pronebird on 25/09/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import BackgroundTasks
import Foundation
import NetworkExtension
import UIKit
import StoreKit
import Logging
import class WireGuardKitTypes.PublicKey

enum TunnelManagerConfiguration {
    /// Delay used before starting to quickly poll the tunnel (in seconds).
    /// Usually when the tunnel is either starting or when reconnecting for a brief moment, until
    /// the tunnel broadcasts the connecting date which is later used to synchronize polling.
    static let tunnelStatusQuickPollDelay: TimeInterval = 1

    /// Poll interval used when connecting date is unknown (in seconds).
    static let tunnelStatusQuickPollInterval: TimeInterval = 3

    /// Delay used for when connecting date is known (in seconds).
    /// Since both GUI and packet tunnel run timers, this accounts for some leeway.
    static let tunnelStatusLongPollDelay: TimeInterval = 0.25

    /// Poll interval used for when connecting date is known (in seconds).
    static let tunnelStatusLongPollInterval = TunnelMonitorConfiguration.connectionTimeout

    /// Private key rotation interval (in seconds).
    static let privateKeyRotationInterval: TimeInterval = 60 * 60 * 24 * 4

    /// Private key rotation retry interval (in seconds).
    static let privateKeyRotationFailureRetryInterval: TimeInterval = 60 * 15
}

/// A class that provides a convenient interface for VPN tunnels configuration, manipulation and
/// monitoring.
final class TunnelManager: TunnelManagerStateDelegate {
    /// Operation categories
    private enum OperationCategory {
        static let manageTunnelProvider = "TunnelManager.manageTunnelProvider"
        static let changeTunnelSettings = "TunnelManager.changeTunnelSettings"
        static let tunnelStateUpdate = "TunnelManager.tunnelStateUpdate"
    }

    static let shared: TunnelManager = {
        return TunnelManager(
            accountsProxy: REST.ProxyFactory.shared.createAccountsProxy(),
            devicesProxy: REST.ProxyFactory.shared.createDevicesProxy()
        )
    }()

    // MARK: - Internal variables

    private let accountsProxy: REST.AccountsProxy
    private let devicesProxy: REST.DevicesProxy

    private let logger = Logger(label: "TunnelManager")
    private let stateQueue = DispatchQueue(label: "TunnelManager.stateQueue")
    private let operationQueue = OperationQueue()
    private let exclusivityController = ExclusivityController()

    private var statusObserver: Tunnel.StatusBlockObserver?
    private var lastMapConnectionStatusOperation: Operation?
    private let observerList = ObserverList<TunnelObserver>()

    private let state: TunnelManager.State

    private var privateKeyRotationTimer: DispatchSourceTimer?
    private var isRunningPeriodicPrivateKeyRotation = false

    private var tunnelStatusPollTimer: DispatchSourceTimer?
    private var isPolling = false
    private var lastConnectingDate: Date?

    var accountNumber: String? {
        return state.tunnelSettings?.account.number
    }

    var accountExpiry: Date? {
        return state.tunnelSettings?.account.expiry
    }

    var isAccountSet: Bool {
        return state.tunnelSettings != nil
    }

    var device: StoredDeviceData? {
        return state.tunnelSettings?.device
    }

    var tunnelSettings: TunnelSettingsV2? {
        return state.tunnelSettings
    }

    var tunnelState: TunnelState {
        return state.tunnelStatus.state
    }

    private init(accountsProxy: REST.AccountsProxy, devicesProxy: REST.DevicesProxy) {
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
        self.state = TunnelManager.State(queue: stateQueue)
        self.state.delegate = self
        self.operationQueue.name = "TunnelManager.operationQueue"
        self.operationQueue.underlyingQueue = stateQueue

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(applicationDidBecomeActive),
            name: UIApplication.didBecomeActiveNotification,
            object: nil
        )
    }

    // MARK: - Periodic private key rotation

    func startPeriodicPrivateKeyRotation() {
        stateQueue.async {
            guard !self.isRunningPeriodicPrivateKeyRotation else { return }

            self.logger.debug("Start periodic private key rotation.")

            self.isRunningPeriodicPrivateKeyRotation = true

            self.updatePrivateKeyRotationTimer()
        }
    }

    func stopPeriodicPrivateKeyRotation() {
        stateQueue.async {
            guard self.isRunningPeriodicPrivateKeyRotation else { return }

            self.logger.debug("Stop periodic private key rotation.")

            self.isRunningPeriodicPrivateKeyRotation = false

            self.privateKeyRotationTimer?.cancel()
            self.privateKeyRotationTimer = nil
        }
    }

    private func updatePrivateKeyRotationTimer() {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        guard self.isRunningPeriodicPrivateKeyRotation else { return }

        if let tunnelSettings = state.tunnelSettings {
            let creationDate = tunnelSettings.device.wgKeyData.creationDate
            let scheduleDate = Date(
                timeInterval: TunnelManagerConfiguration.privateKeyRotationInterval,
                since: creationDate
            )

            schedulePrivateKeyRotationTimer(scheduleDate)
        } else {
            privateKeyRotationTimer?.cancel()
            privateKeyRotationTimer = nil
        }
    }

    /// Schedule new private key rotation timer.
    private func schedulePrivateKeyRotationTimer(_ scheduleDate: Date) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        let timer = DispatchSource.makeTimerSource(queue: stateQueue)

        timer.setEventHandler { [weak self] in
            guard let self = self else { return }

            _ = self.rotatePrivateKey { completion in
                self.stateQueue.async {
                    if let scheduleDate = self.handlePrivateKeyRotationCompletion(completion) {
                        guard self.isRunningPeriodicPrivateKeyRotation else { return }

                        self.schedulePrivateKeyRotationTimer(scheduleDate)
                    }
                }
            }
        }

        // Cancel active timer
        privateKeyRotationTimer?.cancel()

        // Assign new timer
        privateKeyRotationTimer = timer

        // Schedule and activate
        timer.schedule(wallDeadline: .now() + scheduleDate.timeIntervalSinceNow)
        timer.activate()

        logger.debug("Schedule next private key rotation on \(scheduleDate.logFormatDate())")
    }

    // MARK: - Public methods

    func loadConfiguration(completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        let migrateSettingsOperation = MigrateSettingsOperation(
            dispatchQueue: stateQueue,
            accountsProxy: accountsProxy,
            devicesProxy: devicesProxy
        )

        let loadTunnelOperation = LoadTunnelConfigurationOperation(
            dispatchQueue: stateQueue,
            state: state
        ) { [weak self] completion in
            guard let self = self else { return }

            dispatchPrecondition(condition: .onQueue(self.stateQueue))

            if case .failure(let error) = completion {
                self.logger.error(chainedError: error, message: "Failed to load tunnel.")
            }

            self.updatePrivateKeyRotationTimer()

            DispatchQueue.main.async {
                completionHandler(completion.error)
            }
        }

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(
            withName: "Load tunnel configuration"
        ) {
            // no-op
        }

        loadTunnelOperation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(migrateSettingsOperation, categories: [
            OperationCategory.changeTunnelSettings
        ])

        exclusivityController.addOperation(loadTunnelOperation, categories: [
            OperationCategory.manageTunnelProvider,
            OperationCategory.changeTunnelSettings
        ])

        loadTunnelOperation.addDependency(migrateSettingsOperation)

        operationQueue.addOperations([
            migrateSettingsOperation,
            loadTunnelOperation
        ], waitUntilFinished: false)
    }

    func startTunnel() {
        let operation = StartTunnelOperation(
            dispatchQueue: stateQueue,
            state: state,
            encodeErrorHandler: { [weak self] error in
                guard let self = self else { return }

                dispatchPrecondition(condition: .onQueue(self.stateQueue))

                self.logger.error(chainedError: AnyChainedError(error), message: "Failed to encode tunnel options")
            },
            completionHandler: { [weak self] completion in
                guard let self = self else { return }

                dispatchPrecondition(condition: .onQueue(self.stateQueue))

                if case .failure(let error) = completion {
                    self.logger.error(chainedError: error, message: "Failed to start the tunnel.")
                }
            })


        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Start tunnel") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.manageTunnelProvider])

        operationQueue.addOperation(operation)
    }

    func stopTunnel() {
        let operation = StopTunnelOperation(
            dispatchQueue: stateQueue,
            state: state
        ) { [weak self] completion in
            guard let self = self else { return }

            dispatchPrecondition(condition: .onQueue(self.stateQueue))

            guard let error = completion.error else { return }

            // Pass tunnel failure to observers
            DispatchQueue.main.async {
                self.observerList.forEach { observer in
                    observer.tunnelManager(self, didFailWithError: error)
                }
            }
        }

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Stop tunnel") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.manageTunnelProvider])

        operationQueue.addOperation(operation)
    }

    func reconnectTunnel(completionHandler: (() -> Void)?) {
        let operation = ReloadTunnelOperation(queue: stateQueue, state: state) { [weak self] completion in
            guard let self = self else { return }

            dispatchPrecondition(condition: .onQueue(self.stateQueue))

            if let error = completion.error {
                self.logger.error(chainedError: error, message: "Failed to reconnect the tunnel.")
            }

            // Refresh tunnel status only when connecting or reasserting to pick up the next relay,
            // since both states may persist for a long period of time until the tunnel is fully
            // connected.
            switch self.tunnelState {
            case .connecting, .reconnecting:
                self.logger.debug("Refresh tunnel status due to reconnect.")
                self.refreshTunnelStatus()

            default:
                break
            }

            DispatchQueue.main.async {
                completionHandler?()
            }
        }

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Reconnect tunnel") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.manageTunnelProvider])

        operationQueue.addOperation(operation)
    }

    func setAccount(action: SetAccountAction, completionHandler: @escaping (OperationCompletion<StoredAccountData?, TunnelManager.Error>) -> Void) {
        let operation = SetAccountOperation(
            dispatchQueue: stateQueue,
            state: state,
            accountsProxy: accountsProxy,
            devicesProxy: devicesProxy,
            action: action,
            willDeleteVPNConfigurationHandler: { [weak self] in
                guard let self = self else { return }

                dispatchPrecondition(condition: .onQueue(self.stateQueue))

                // Unregister from receiving VPN connection status changes
                self.unsubscribeVPNStatusObserver()

                // Cancel last VPN status mapping operation
                self.lastMapConnectionStatusOperation?.cancel()
                self.lastMapConnectionStatusOperation = nil
            },
            completionHandler: { [weak self] completion in
                guard let self = self else { return }

                dispatchPrecondition(condition: .onQueue(self.stateQueue))

                self.updatePrivateKeyRotationTimer()

                DispatchQueue.main.async {
                    completionHandler(completion)
                }
            })

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: action.taskName) {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [
            OperationCategory.manageTunnelProvider,
            OperationCategory.changeTunnelSettings
        ])

        operationQueue.addOperation(operation)
    }

    func unsetAccount(completionHandler: @escaping () -> Void) {
        setAccount(action: .unset) { _ in
            completionHandler()
        }
    }

    func updateAccountData(_ completionHandler: ((TunnelManager.Error?) -> Void)? = nil) {
        let operation = UpdateAccountDataOperation(
            dispatchQueue: stateQueue,
            state: state,
            accountsProxy: accountsProxy
        )

        operation.completionQueue = .main
        operation.completionHandler = { completion in
            completionHandler?(completion.error)
        }

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Update account data") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [
            OperationCategory.changeTunnelSettings
        ])

        operationQueue.addOperation(operation)
    }

    func updateDeviceData(_ completionHandler: @escaping (OperationCompletion<StoredDeviceData, TunnelManager.Error>) -> Void) -> Cancellable {
        let operation = UpdateDeviceDataOperation(
            dispatchQueue: stateQueue,
            state: state,
            devicesProxy: devicesProxy
        )

        operation.completionQueue = .main
        operation.completionHandler = completionHandler

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Update device data") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [
            OperationCategory.changeTunnelSettings
        ])

        operationQueue.addOperation(operation)

        return operation
    }

    func regeneratePrivateKey(completionHandler: ((TunnelManager.Error?) -> Void)? = nil) {
        let operation = RotateKeyOperation(
            dispatchQueue: stateQueue,
            state: state,
            devicesProxy: devicesProxy,
            rotationInterval: nil
        ) { [weak self] completion in
            guard let self = self else { return }

            dispatchPrecondition(condition: .onQueue(self.stateQueue))

            switch completion {
            case .success:
                self.updatePrivateKeyRotationTimer()
                self.reconnectTunnel(completionHandler: nil)

            case .failure(let error):
                self.logger.error(chainedError: error, message: "Failed to regenerate private key.")

            case .cancelled:
                break
            }

            DispatchQueue.main.async {
                completionHandler?(completion.error)
            }
        }

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Regenerate private key") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.changeTunnelSettings])

        operationQueue.addOperation(operation)
    }

    func rotatePrivateKey(completionHandler: @escaping (OperationCompletion<KeyRotationResult, TunnelManager.Error>) -> Void) -> Cancellable {
        let operation = RotateKeyOperation(
            dispatchQueue: stateQueue,
            state: state,
            devicesProxy: devicesProxy,
            rotationInterval: TunnelManagerConfiguration.privateKeyRotationInterval
        ) { [weak self] completion in
            guard let self = self else { return }

            dispatchPrecondition(condition: .onQueue(self.stateQueue))

            switch completion {
            case .success:
                self.reconnectTunnel {
                    completionHandler(completion)
                }

            case .failure(let error):
                self.logger.error(chainedError: error, message: "Failed to rotate private key.")

                DispatchQueue.main.async {
                    completionHandler(completion)
                }

            case .cancelled:
                DispatchQueue.main.async {
                    completionHandler(completion)
                }
            }
        }

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Rotate private key") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.changeTunnelSettings])

        operationQueue.addOperation(operation)

        return operation
    }

    func setRelayConstraints(_ newConstraints: RelayConstraints, completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        scheduleTunnelSettingsUpdate(
            taskName: "Set relay constraints",
            modificationBlock: { tunnelSettings in
                tunnelSettings.relayConstraints = newConstraints
            },
            completionHandler: completionHandler
        )
    }

    func setDNSSettings(_ newDNSSettings: DNSSettings, completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        scheduleTunnelSettingsUpdate(
            taskName: "Set DNS settings",
            modificationBlock: { tunnelSettings in
                tunnelSettings.dnsSettings = newDNSSettings
            },
            completionHandler: completionHandler
        )
    }

    // MARK: - Tunnel observeration

    /// Add tunnel observer.
    /// In order to cancel the observation, either call `removeObserver(_:)` or simply release
    /// the observer.
    func addObserver(_ observer: TunnelObserver) {
        observerList.append(observer)
    }

    /// Remove tunnel observer.
    func removeObserver(_ observer: TunnelObserver) {
        observerList.remove(observer)
    }

    // MARK: - TunnelManagerStateDelegate

    func tunnelManagerState(_ state: TunnelManager.State, didChangeTunnelSettings newTunnelSettinggs: TunnelSettingsV2?) {
        DispatchQueue.main.async {
            self.observerList.forEach { observer in
                observer.tunnelManager(self, didUpdateTunnelSettings: newTunnelSettinggs)
            }
        }
    }

    func tunnelManagerState(_ state: TunnelManager.State, didChangeTunnelStatus newTunnelStatus: TunnelStatus) {
        logger.info("Status: \(newTunnelStatus).")

        switch newTunnelStatus.state {
        case .connecting, .reconnecting:
            // Start polling tunnel status to keep the relay information up to date
            // while the tunnel process is trying to connect.
            startPollingTunnelStatus(connectingDate: newTunnelStatus.connectingDate)

        case .pendingReconnect, .connected, .disconnecting, .disconnected:
            // Stop polling tunnel status once connection moved to final state.
            cancelPollingTunnelStatus()
        }

        DispatchQueue.main.async {
            self.observerList.forEach { (observer) in
                observer.tunnelManager(self, didUpdateTunnelState: newTunnelStatus.state)
            }
        }
    }

    func tunnelManagerState(_ state: TunnelManager.State, didChangeTunnelProvider newTunnelObject: Tunnel?, shouldRefreshTunnelState: Bool) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        // Register for tunnel connection status changes
        if let newTunnelObject = newTunnelObject {
            subscribeVPNStatusObserver(tunnel: newTunnelObject)
        } else {
            unsubscribeVPNStatusObserver()
        }

        // Update the existing state
        if shouldRefreshTunnelState {
            logger.debug("Refresh tunnel status for new tunnel.")
            refreshTunnelStatus()
        }
    }

    // MARK: - Private methods

    private func subscribeVPNStatusObserver(tunnel: Tunnel) {
        unsubscribeVPNStatusObserver()

        statusObserver = tunnel.addBlockObserver(queue: stateQueue) { [weak self] tunnel, status in
            guard let self = self else { return }

            self.logger.debug("VPN connection status changed to \(status).")
            self.updateTunnelStatus(status)
        }
    }

    private func unsubscribeVPNStatusObserver() {
        statusObserver?.invalidate()
        statusObserver = nil
    }

    private func refreshTunnelStatus() {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        if let connectionStatus = self.state.tunnel?.status {
            updateTunnelStatus(connectionStatus)
        }
    }

    /// Update `TunnelStatus` from `NEVPNStatus`.
    /// Collects the `PacketTunnelStatus` from the tunnel via IPC if needed before assigning
    /// the `tunnelStatus`.
    private func updateTunnelStatus(_ connectionStatus: NEVPNStatus) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        let operation = MapConnectionStatusOperation(
            queue: stateQueue,
            state: state,
            connectionStatus: connectionStatus
        ) { [weak self] in
            guard let self = self else { return }

            dispatchPrecondition(condition: .onQueue(self.stateQueue))

            self.startTunnel()
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.tunnelStateUpdate])

        // Cancel last VPN status mapping operation
        lastMapConnectionStatusOperation?.cancel()
        lastMapConnectionStatusOperation = operation

        operationQueue.addOperation(operation)
    }

    @objc private func applicationDidBecomeActive() {
        stateQueue.async {
            self.logger.debug("Refresh tunnel status due to application becoming active.")
            self.refreshTunnelStatus()
        }
    }

    fileprivate func scheduleTunnelSettingsUpdate(taskName: String, modificationBlock: @escaping (inout TunnelSettingsV2) -> Void, completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        let operation = ResultBlockOperation<Void, TunnelManager.Error>(
            dispatchQueue: stateQueue
        ) { operation in
            guard var tunnelSettings = self.tunnelSettings else {
                operation.finish(completion: .failure(.unsetAccount))
                return
            }

            do {
                modificationBlock(&tunnelSettings)

                try SettingsManager.writeSettings(tunnelSettings)

                self.state.tunnelSettings = tunnelSettings
                self.reconnectTunnel(completionHandler: nil)

                operation.finish(completion: .success(()))
            } catch {
                self.logger.error(
                    chainedError: AnyChainedError(error),
                    message: "Failed to write settings."
                )

                operation.finish(completion: .failure(.writeSettings(error)))
            }
        }

        operation.completionQueue = .main
        operation.completionHandler = { completion in
            completionHandler(completion.error)
        }

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: taskName) {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.changeTunnelSettings])

        operationQueue.addOperation(operation)
    }

    // MARK: - Tunnel status polling.

    private func computeNextPollDateAndRepeatInterval(connectingDate: Date?) -> (Date, TimeInterval) {
        let delay, repeating: TimeInterval
        let fireDate: Date

        if let connectingDate = connectingDate {
            // Compute the schedule date for timer relative to when the packet tunnel started
            // connecting.
            delay = TunnelManagerConfiguration.tunnelStatusLongPollDelay
            repeating = TunnelManagerConfiguration.tunnelStatusLongPollInterval

            // Compute the time elapsed since connecting date.
            let elapsed = max(0, Date().timeIntervalSince(connectingDate))

            // Compute how many times the timer has fired so far.
            let fireCount = floor(elapsed / repeating)

            // Compute when the timer will fire next time.
            let nextDelta = (fireCount + 1) * repeating

            // Compute the fire date adding extra delay to account for leeway.
            fireDate = connectingDate.addingTimeInterval(nextDelta + delay)
        } else {
            // Do quick polling until it's known when the packet tunnel started connecting.
            delay = TunnelManagerConfiguration.tunnelStatusQuickPollDelay
            repeating = TunnelManagerConfiguration.tunnelStatusQuickPollInterval

            fireDate = Date(timeIntervalSinceNow: delay)
        }

        return (fireDate, repeating)
    }

    private func startPollingTunnelStatus(connectingDate: Date?) {
        guard lastConnectingDate != connectingDate || !isPolling else { return }

        lastConnectingDate = connectingDate
        isPolling = true

        let (fireDate, repeating) = computeNextPollDateAndRepeatInterval(connectingDate: connectingDate)
        logger.debug("Start polling tunnel status at \(fireDate.logFormatDate()) every \(repeating) second(s).")

        let timer = DispatchSource.makeTimerSource(queue: stateQueue)
        timer.setEventHandler { [weak self] in
            guard let self = self else { return }

            self.logger.debug("Refresh tunnel status (poll).")
            self.refreshTunnelStatus()
        }

        timer.schedule(
            wallDeadline: .now() + fireDate.timeIntervalSinceNow,
            repeating: repeating
        )

        timer.resume()

        tunnelStatusPollTimer?.cancel()
        tunnelStatusPollTimer = timer
    }

    private func cancelPollingTunnelStatus() {
        guard isPolling else { return }

        logger.debug("Cancel tunnel status polling.")

        tunnelStatusPollTimer?.cancel()
        tunnelStatusPollTimer = nil
        lastConnectingDate = nil
        isPolling = false
    }

}

extension TunnelManager {
    /// Key rotation result.
    enum KeyRotationResult: CustomStringConvertible {
        /// Request to rotate the key was throttled.
        case throttled(_ lastKeyCreationDate: Date)

        /// New key was generated.
        case finished

        var description: String {
            switch self {
            case .throttled:
                return "throttled"
            case .finished:
                return "finished"
            }
        }
    }
}

// MARK: - Background tasks

@available(iOS 13.0, *)
extension TunnelManager {

    /// Register background task with scheduler.
    func registerBackgroundTask() {
        let taskIdentifier = ApplicationConfiguration.privateKeyRotationTaskIdentifier

        let isRegistered = BGTaskScheduler.shared.register(forTaskWithIdentifier: taskIdentifier, using: nil) { task in
            self.handleBackgroundTask(task as! BGProcessingTask)
        }

        if isRegistered {
            logger.debug("Registered private key rotation task")
        } else {
            logger.error("Failed to register private key rotation task")
        }
    }

    /// Schedule background task relative to the private key creation date.
    func scheduleBackgroundTask() throws {
        guard let tunnelSettings = state.tunnelSettings else {
            throw Error.unsetAccount
        }

        let creationDate = tunnelSettings.device.wgKeyData.creationDate
        let beginDate = Date(
            timeInterval: TunnelManagerConfiguration.privateKeyRotationInterval,
            since: creationDate
        )

        return try submitBackgroundTask(at: beginDate)
    }

    /// Create and submit task request to scheduler.
    private func submitBackgroundTask(at beginDate: Date) throws {
        let taskIdentifier = ApplicationConfiguration.privateKeyRotationTaskIdentifier

        let request = BGProcessingTaskRequest(identifier: taskIdentifier)
        request.earliestBeginDate = beginDate
        request.requiresNetworkConnectivity = true

        try BGTaskScheduler.shared.submit(request)
    }

    /// Background task handler.
    private func handleBackgroundTask(_ task: BGProcessingTask) {
        logger.debug("Start private key rotation task")

        let cancellableTask = rotatePrivateKey { completion in
            if let scheduleDate = self.handlePrivateKeyRotationCompletion(completion) {
                // Schedule next background task
                do {
                    try self.submitBackgroundTask(at: scheduleDate)

                    self.logger.debug(
                        "Scheduled next private key rotation task at \(scheduleDate.logFormatDate())"
                    )
                } catch {
                    self.logger.error(
                        chainedError: AnyChainedError(error),
                        message: "Failed to schedule next private key rotation task."
                    )
                }
            }

            // Complete current task
            task.setTaskCompleted(success: completion.isSuccess)
        }

        task.expirationHandler = {
            cancellableTask.cancel()
        }
    }
}

extension TunnelManager {
    fileprivate func handlePrivateKeyRotationCompletion(_ completion: OperationCompletion<KeyRotationResult, TunnelManager.Error>) -> Date? {
        switch completion {
        case .success(let result):
            switch result {
            case .finished:
                logger.debug("Finished private key rotation.")
            case .throttled:
                logger.debug("Private key was already rotated earlier.")
            }

            return nextScheduleDate(result)

        case .failure(let error):
            logger.error(chainedError: error, message: "Failed to rotate private key.")

            return nextRetryScheduleDate(error)

        case .cancelled:
            logger.debug("Private key rotation was cancelled.")

            return Date(timeIntervalSinceNow: TunnelManagerConfiguration.privateKeyRotationFailureRetryInterval)
        }
    }

    fileprivate func nextScheduleDate(_ result: KeyRotationResult) -> Date {
        switch result {
        case .finished:
            return Date(timeIntervalSinceNow: TunnelManagerConfiguration.privateKeyRotationInterval)

        case .throttled(let lastKeyCreationDate):
            return Date(timeInterval: TunnelManagerConfiguration.privateKeyRotationInterval, since: lastKeyCreationDate)
        }
    }

    fileprivate func nextRetryScheduleDate(_ error: TunnelManager.Error) -> Date? {
        switch error {
        case .unsetAccount:
            // Do not retry if logged out.
            return nil

        case .rotateKey(.unhandledResponse(_, let serverErrorResponse))
            where serverErrorResponse?.code == .invalidAccount ||
            serverErrorResponse?.code == .deviceNotFound:
            // Do not retry if account or device were removed.
            return nil

        default:
            return Date(timeIntervalSinceNow: TunnelManagerConfiguration.privateKeyRotationFailureRetryInterval)
        }
    }
}

extension TunnelManager: AppStorePaymentObserver {
    func appStorePaymentManager(_ manager: AppStorePaymentManager,
                                transaction: SKPaymentTransaction?,
                                payment: SKPayment,
                                accountToken: String?,
                                didFailWithError error: AppStorePaymentManager.Error
    )
    {
        // no-op
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager,
                                transaction: SKPaymentTransaction,
                                accountToken: String,
                                didFinishWithResponse response: REST.CreateApplePaymentResponse
    )
    {
        scheduleTunnelSettingsUpdate(
            taskName: "Update account expiry after in-app purchase",
            modificationBlock: { tunnelSettings in
                if tunnelSettings.account.number == accountToken {
                    tunnelSettings.account.expiry = response.newExpiry
                }
            },
            completionHandler: { error in
                guard let error = error else { return }

                self.logger.error(
                    chainedError: error,
                    message: "Failed to update account expiry after in-app purchase"
                )
            }
        )
    }
}
