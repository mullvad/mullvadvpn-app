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
        return TunnelManager(restClient: REST.Client.shared)
    }()

    // MARK: - Internal variables

    private let restClient: REST.Client

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

    var tunnelInfo: TunnelInfo? {
        return state.tunnelInfo
    }

    var tunnelState: TunnelState {
        return state.tunnelStatus.state
    }

    private init(restClient: REST.Client) {
        self.restClient = restClient
        self.state = TunnelManager.State(queue: stateQueue)
        self.state.delegate = self

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

        if let tunnelInfo = self.state.tunnelInfo {
            let creationDate = tunnelInfo.tunnelSettings.interface.privateKey.creationDate
            let scheduleDate = Date(timeInterval: TunnelManagerConfiguration.privateKeyRotationInterval, since: creationDate)

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

            _ = self.rotatePrivateKey { rotationResult, error in
                self.stateQueue.async {
                    if let scheduleDate = self.handlePrivateKeyRotationCompletion(result: rotationResult, error: error) {
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

    /// Initialize the TunnelManager with the tunnel from the system.
    ///
    /// The given account token is used to ensure that the system tunnel was configured for the same
    /// account. The system tunnel is removed in case of inconsistency.
    func loadTunnel(accountToken: String?, completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        let operation = LoadTunnelOperation(queue: stateQueue, state: state, accountToken: accountToken) { [weak self] completion in
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

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Load tunnel") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.manageTunnelProvider, OperationCategory.changeTunnelSettings])

        operationQueue.addOperation(operation)
    }

    func startTunnel() {
        let operation = StartTunnelOperation(
            queue: stateQueue,
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
        let operation = StopTunnelOperation(queue: stateQueue, state: state) { [weak self] completion in
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

            // Refresh tunnel status since reasserting may not be lowered until the tunnel is fully
            // connected.
            self.logger.debug("Refresh tunnel status due to reconnect.")
            self.refreshTunnelStatus()

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

    func setAccount(accountToken: String, completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        let operation = makeSetAccountOperation(accountToken: accountToken) { completion in
            DispatchQueue.main.async {
                completionHandler(completion.error)
            }
        }

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Set tunnel account") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.manageTunnelProvider, OperationCategory.changeTunnelSettings])

        operationQueue.addOperation(operation)
    }

    func unsetAccount(completionHandler: @escaping () -> Void) {
        let operation = makeSetAccountOperation(accountToken: nil) { _ in
            DispatchQueue.main.async {
                completionHandler()
            }
        }

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Unset tunnel account") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.manageTunnelProvider, OperationCategory.changeTunnelSettings])

        operationQueue.addOperation(operation)
    }

    func regeneratePrivateKey(completionHandler: ((TunnelManager.Error?) -> Void)? = nil) {
        let operation = ReplaceKeyOperation.operationForKeyRegeneration(queue: stateQueue, state: state, restClient: restClient) { [weak self] completion in
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

    func rotatePrivateKey(completionHandler: @escaping (KeyRotationResult?, TunnelManager.Error?) -> Void) -> Cancellable {
        let operation = ReplaceKeyOperation.operationForKeyRotation(
            queue: stateQueue,
            state: state,
            restClient: restClient,
            rotationInterval: TunnelManagerConfiguration.privateKeyRotationInterval) { [weak self] completion in
                guard let self = self else { return }

                dispatchPrecondition(condition: .onQueue(self.stateQueue))

                var rotationResult: KeyRotationResult?
                var rotationError: TunnelManager.Error?

                switch completion {
                case .success(let result):
                    rotationResult = result

                    self.reconnectTunnel {
                        completionHandler(rotationResult, rotationError)
                    }

                case .failure(let error):
                    rotationError = error
                    self.logger.error(chainedError: error, message: "Failed to rotate private key.")

                    DispatchQueue.main.async {
                        completionHandler(rotationResult, rotationError)
                    }

                case .cancelled:
                    DispatchQueue.main.async {
                        completionHandler(rotationResult, rotationError)
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
                tunnelSettings.interface.dnsSettings = newDNSSettings
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

    func tunnelManagerState(_ state: TunnelManager.State, didChangeTunnelInfo newTunnelInfo: TunnelInfo?) {
        DispatchQueue.main.async {
            self.observerList.forEach { (observer) in
                observer.tunnelManager(self, didUpdateTunnelSettings: newTunnelInfo)
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

        let operation = MapConnectionStatusOperation(queue: stateQueue, state: state, connectionStatus: connectionStatus) { [weak self] in
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

    private func makeSetAccountOperation(accountToken: String?, completionHandler: @escaping (OperationCompletion<(), TunnelManager.Error>) -> Void) -> Operation {
        return SetAccountOperation(
            queue: stateQueue,
            state: state,
            restClient: restClient,
            accountToken: accountToken,
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

                completionHandler(completion)
            })
    }

    private func scheduleTunnelSettingsUpdate(taskName: String, modificationBlock: @escaping (inout TunnelSettings) -> Void, completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        let operation = SetTunnelSettingsOperation(
            queue: stateQueue,
            state: state,
            modificationBlock: modificationBlock,
            completionHandler: { [weak self] completion in
                guard let self = self else { return }

                dispatchPrecondition(condition: .onQueue(self.stateQueue))

                switch completion {
                case .success:
                    self.reconnectTunnel(completionHandler: nil)

                case .failure(let error):
                    self.logger.error(chainedError: error, message: "Failed to set tunnel settings.")

                case .cancelled:
                    break
                }

                DispatchQueue.main.async {
                    completionHandler(completion.error)
                }
            })

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
    func scheduleBackgroundTask() -> Result<(), TunnelManager.Error> {
        if let tunnelInfo = self.state.tunnelInfo {
            let creationDate = tunnelInfo.tunnelSettings.interface.privateKey.creationDate
            let beginDate = Date(timeInterval: TunnelManagerConfiguration.privateKeyRotationInterval, since: creationDate)

            return submitBackgroundTask(at: beginDate)
        } else {
            return .failure(.unsetAccount)
        }
    }

    /// Create and submit task request to scheduler.
    private func submitBackgroundTask(at beginDate: Date) -> Result<(), TunnelManager.Error> {
        let taskIdentifier = ApplicationConfiguration.privateKeyRotationTaskIdentifier

        let request = BGProcessingTaskRequest(identifier: taskIdentifier)
        request.earliestBeginDate = beginDate
        request.requiresNetworkConnectivity = true

        return Result { try BGTaskScheduler.shared.submit(request) }
            .mapError { error in
                return .backgroundTaskScheduler(error)
            }
    }

    /// Background task handler.
    private func handleBackgroundTask(_ task: BGProcessingTask) {
        logger.debug("Start private key rotation task")

        let request = rotatePrivateKey { rotationResult, error in
            if let scheduleDate = self.handlePrivateKeyRotationCompletion(result: rotationResult, error: error) {
                // Schedule next background task
                switch self.submitBackgroundTask(at: scheduleDate) {
                case .success:
                    self.logger.debug("Scheduled next private key rotation task at \(scheduleDate.logFormatDate())")

                case .failure(let error):
                    self.logger.error(chainedError: error, message: "Failed to schedule next private key rotation task.")
                }
            }

            // Complete current task
            task.setTaskCompleted(success: error == nil)
        }

        task.expirationHandler = {
            request.cancel()
        }
    }
}

extension TunnelManager {
    fileprivate func handlePrivateKeyRotationCompletion(result: KeyRotationResult?, error: TunnelManager.Error?) -> Date? {
        if let error = error {
            logger.error(chainedError: error, message: "Failed to rotate private key")

            return nextRetryScheduleDate(error)
        } else if let result = result {
            switch result {
            case .finished:
                logger.debug("Finished private key rotation")
            case .throttled:
                logger.debug("Private key was already rotated earlier")
            }

            return nextScheduleDate(result)
        } else {
            logger.debug("Private key rotation was cancelled")

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

        case .replaceWireguardKey(.server(.invalidAccount)):
            // Do not retry if account was removed.
            return nil

        default:
            return Date(timeIntervalSinceNow: TunnelManagerConfiguration.privateKeyRotationFailureRetryInterval)
        }
    }
}
