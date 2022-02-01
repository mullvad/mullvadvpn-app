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
import class WireGuardKit.PublicKey

/// A class that provides a convenient interface for VPN tunnels configuration, manipulation and
/// monitoring.
class TunnelManager: TunnelManagerStateDelegate
{
    /// Private key rotation interval (in seconds)
    private static let privateKeyRotationInterval: TimeInterval = 60 * 60 * 24 * 4

    /// Private key rotation retry interval (in seconds)
    private static let privateKeyRotationFailureRetryInterval: TimeInterval = 60 * 15

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

    private var lastMapConnectionStatusOperation: Operation?
    private let observerList = ObserverList<AnyTunnelObserver>()

    private let state: TunnelManager.State

    var tunnelInfo: TunnelInfo? {
        return state.tunnelInfo
    }

    var tunnelState: TunnelState {
        return state.tunnelState
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

    private var privateKeyRotationTimer: DispatchSourceTimer?
    private var isRunningPeriodicPrivateKeyRotation = false

    func startPeriodicPrivateKeyRotation() {
        stateQueue.async {
            guard !self.isRunningPeriodicPrivateKeyRotation else { return }

            self.logger.debug("Start periodic private key rotation")

            self.isRunningPeriodicPrivateKeyRotation = true

            self.updatePrivateKeyRotationTimer()
        }
    }

    func stopPeriodicPrivateKeyRotation() {
        stateQueue.async {
            guard self.isRunningPeriodicPrivateKeyRotation else { return }

            self.logger.debug("Stop periodic private key rotation")

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
            let scheduleDate = Date(timeInterval: Self.privateKeyRotationInterval, since: creationDate)

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

            self.rotatePrivateKey { rotationResult, error in
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
                self.logger.error(chainedError: error, message: "Failed to load tunnel")
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
                    self.logger.error(chainedError: error)
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
                self.logger.error(chainedError: error)
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
                self.logger.error(chainedError: error, message: "Failed to regenerate private key")

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

    func rotatePrivateKey(completionHandler: @escaping (KeyRotationResult?, TunnelManager.Error?) -> Void) {
        let operation = ReplaceKeyOperation.operationForKeyRotation(
            queue: stateQueue,
            state: state,
            restClient: restClient,
            rotationInterval: Self.privateKeyRotationInterval) { [weak self] completion in
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
                    self.logger.error(chainedError: error, message: "Failed to rotate private key")

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
    func addObserver<T: TunnelObserver>(_ observer: T) {
        observerList.append(AnyTunnelObserver(observer))
    }

    /// Remove tunnel observer.
    func removeObserver<T: TunnelObserver>(_ observer: T) {
        observerList.remove(AnyTunnelObserver(observer))
    }

    // MARK: - TunnelManagerStateDelegate

    func tunnelManagerState(_ state: TunnelManager.State, didChangeTunnelInfo newTunnelInfo: TunnelInfo?) {
        DispatchQueue.main.async {
            self.observerList.forEach { (observer) in
                observer.tunnelManager(self, didUpdateTunnelSettings: newTunnelInfo)
            }
        }
    }

    func tunnelManagerState(_ state: TunnelManager.State, didChangeTunnelState newTunnelState: TunnelState) {
        logger.info("Set tunnel state: \(newTunnelState)")

        DispatchQueue.main.async {
            self.observerList.forEach { (observer) in
                observer.tunnelManager(self, didUpdateTunnelState: newTunnelState)
            }
        }
    }

    func tunnelManagerState(_ state: TunnelManager.State, didChangeTunnelProvider newTunnelProvider: TunnelProviderManagerType?, shouldRefreshTunnelState: Bool) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        // Register for tunnel connection status changes
        if let newTunnelProvider = newTunnelProvider {
            subscribeVPNStatusObserver(for: newTunnelProvider)
        } else {
            unsubscribeVPNStatusObserver()
        }

        // Update the existing state
        if shouldRefreshTunnelState {
            updateTunnelState()
        }
    }

    // MARK: - Private methods

    private func subscribeVPNStatusObserver(for tunnelProvider: TunnelProviderManagerType) {
        unsubscribeVPNStatusObserver()

        NotificationCenter.default.addObserver(
            self, selector: #selector(didReceiveVPNStatusChange(_:)),
            name: .NEVPNStatusDidChange,
            object: tunnelProvider.connection
        )
    }

    private func unsubscribeVPNStatusObserver() {
        NotificationCenter.default.removeObserver(self, name: .NEVPNStatusDidChange, object: nil)
    }

    @objc private func didReceiveVPNStatusChange(_ notification: Notification) {
        stateQueue.async {
            self.updateTunnelState()
        }
    }

    /// Update `TunnelState` from `NEVPNStatus`.
    /// Collects the `TunnelConnectionInfo` from the tunnel via IPC if needed before assigning the `tunnelState`
    private func updateTunnelState() {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        guard let connectionStatus = self.state.tunnelProvider?.connection.status else { return }

        logger.debug("VPN status changed to \(connectionStatus)")

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
            // Refresh tunnel state when application becomes active.
            self.updateTunnelState()
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
                    self.logger.error(chainedError: error, message: "Failed to set tunnel settings")

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
            let beginDate = Date(timeInterval: Self.privateKeyRotationInterval, since: creationDate)

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

        rotatePrivateKey { rotationResult, error in
            if let scheduleDate = self.handlePrivateKeyRotationCompletion(result: rotationResult, error: error) {
                // Schedule next background task
                switch self.submitBackgroundTask(at: scheduleDate) {
                case .success:
                    self.logger.debug("Scheduled next private key rotation task at \(scheduleDate.logFormatDate())")

                case .failure(let error):
                    self.logger.error(chainedError: error, message: "Failed to schedule next private key rotation task")
                }
            }

            // Complete current task
            task.setTaskCompleted(success: error == nil)
        }

        task.expirationHandler = {
            // TODO: handle cancellation?
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

            return Date(timeIntervalSinceNow: Self.privateKeyRotationFailureRetryInterval)
        }
    }

    fileprivate func nextScheduleDate(_ result: KeyRotationResult) -> Date {
        switch result {
        case .finished:
            return Date(timeIntervalSinceNow: Self.privateKeyRotationInterval)

        case .throttled(let lastKeyCreationDate):
            return Date(timeInterval: Self.privateKeyRotationInterval, since: lastKeyCreationDate)
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
            return Date(timeIntervalSinceNow: Self.privateKeyRotationFailureRetryInterval)
        }
    }
}
