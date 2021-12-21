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

// Switch to stabs on simulator
#if targetEnvironment(simulator)
typealias TunnelProviderManagerType = SimulatorTunnelProviderManager
#else
typealias TunnelProviderManagerType = NETunnelProviderManager
#endif

/// A class that provides a convenient interface for VPN tunnels configuration, manipulation and
/// monitoring.
class TunnelManager: StartTunnelOperationDelegate, StopTunnelOperationDelegate,
                     ReloadTunnelOperationDelegate, MapConnectionStatusOperationDelegate,
                     RenegeratePrivateKeyOperationDelegate, RotatePrivateKeyOperationDelegate,
                     LoadTunnelOperationDelegate, SetAccountOperationDelegate, UnsetAccountOperationDelegate,
                     SetTunnelSettingsOperationDelegate
{
    /// Private key rotation interval (in seconds)
    private static let privateKeyRotationInterval: TimeInterval = 60 * 60 * 24 * 4

    /// Private key rotation retry interval (in seconds)
    private static let privateKeyRotationFailureRetryInterval: TimeInterval = 60 * 15

    /// Operation categories
    private enum OperationCategory {
        static let manageTunnelProvider = "TunnelManager.manageTunnelProvider"
        static let changeTunnelSettings = "TunnelManager.changeTunnelSettings"
        static let notifyTunnelSettingsChange = "TunnelManager.notifyTunnelSettingsChange"
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
    private var tunnelProvider: TunnelProviderManagerType?

    private let stateLock = NSLock()
    private let observerList = ObserverList<AnyTunnelObserver>()

    private(set) var tunnelInfo: TunnelInfo? {
        set {
            stateLock.withCriticalBlock {
                _tunnelInfo = newValue
                tunnelInfoDidChange(newValue)
            }
        }
        get {
            return stateLock.withCriticalBlock {
                return _tunnelInfo
            }
        }
    }

    private var _tunnelInfo: TunnelInfo?
    private var _tunnelState = TunnelState.disconnected

    private(set) var tunnelState: TunnelState {
        set {
            stateLock.withCriticalBlock {
                guard _tunnelState != newValue else { return }

                _tunnelState = newValue

                tunnelStateDidChange(newValue)
            }
        }
        get {
            return stateLock.withCriticalBlock {
                return _tunnelState
            }
        }
    }


    private init(restClient: REST.Client) {
        self.restClient = restClient

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

        if let tunnelInfo = self.tunnelInfo {
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
        let operation = LoadTunnelOperation(queue: stateQueue, token: accountToken, delegate: self) { error in
            DispatchQueue.main.async {
                completionHandler(error)
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
        let operation = StartTunnelOperation(queue: stateQueue, delegate: self)

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
        let operation = StopTunnelOperation(queue: stateQueue, delegate: self)

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
        let operation = ReloadTunnelOperation(queue: stateQueue, delegate: self)

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Reconnect tunnel") {
            operation.cancel()
        }

        operation.completionBlock = {
            DispatchQueue.main.async {
                completionHandler?()
            }

            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.manageTunnelProvider])

        operationQueue.addOperation(operation)
    }

    func setAccount(accountToken: String, completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        let operation = SetAccountOperation(queue: stateQueue, restClient: restClient, token: accountToken, delegate: self) { error in
            DispatchQueue.main.async {
                completionHandler(error)
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

    /// Remove the account token and remove the active tunnel
    func unsetAccount(completionHandler: @escaping () -> Void) {
        let operation = UnsetAccountOperation(queue: stateQueue, restClient: restClient, delegate: self)

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Unset tunnel account") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)

            DispatchQueue.main.async {
                completionHandler()
            }
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.manageTunnelProvider, OperationCategory.changeTunnelSettings])

        operationQueue.addOperation(operation)
    }

    func regeneratePrivateKey(completionHandler: ((TunnelManager.Error?) -> Void)? = nil) {
        let operation = RenegeratePrivateKeyOperation(
            queue: stateQueue,
            restClient: restClient,
            delegate: self) { error in
                DispatchQueue.main.async {
                    completionHandler?(error)
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
        let operation = RotatePrivateKeyOperation(queue: stateQueue, restClient: restClient, rotationInterval: Self.privateKeyRotationInterval, delegate: self) { rotationResult, error in
            self.reconnectTunnel {
                completionHandler(rotationResult, error)
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
        let operation = SetTunnelSettingsOperation(
            queue: stateQueue,
            delegate: self,
            modificationBlock: { tunnelSettings in
                tunnelSettings.relayConstraints = newConstraints
            },
            completionHandler: { error in
                DispatchQueue.main.async {
                    completionHandler(error)
                }
            })

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Set relay constraints") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.changeTunnelSettings])

        operationQueue.addOperation(operation)
    }

    func setDNSSettings(_ newDNSSettings: DNSSettings, completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        let operation = SetTunnelSettingsOperation(
            queue: stateQueue,
            delegate: self,
            modificationBlock: { tunnelSettings in
                tunnelSettings.interface.dnsSettings = newDNSSettings
            },
            completionHandler: { error in
                DispatchQueue.main.async {
                    completionHandler(error)
                }
            })

        let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Set DNS settings") {
            operation.cancel()
        }

        operation.completionBlock = {
            UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
        }

        exclusivityController.addOperation(operation, categories: [OperationCategory.changeTunnelSettings])

        operationQueue.addOperation(operation)
    }

    // MARK: - Tunnel observeration

    /// Add tunnel observer.
    /// In order to cancel the observation, either call `removeTunnelObserver(_:)` or simply release
    /// the observer.
    func addObserver<T: TunnelObserver>(_ observer: T) {
        observerList.append(AnyTunnelObserver(observer))
    }

    /// Remove tunnel observer.
    func removeObserver<T: TunnelObserver>(_ observer: T) {
        observerList.remove(AnyTunnelObserver(observer))
    }

    // MARK: - Private methods

    private func tunnelInfoDidChange(_ newTunnelInfo: TunnelInfo?) {
        DispatchQueue.main.async {
            self.observerList.forEach { (observer) in
                observer.tunnelManager(self, didUpdateTunnelSettings: newTunnelInfo)
            }
        }
    }

    private func tunnelStateDidChange(_ newTunnelState: TunnelState) {
        logger.info("Set tunnel state: \(newTunnelState)")

        DispatchQueue.main.async {
            self.observerList.forEach { (observer) in
                observer.tunnelManager(self, didUpdateTunnelState: newTunnelState)
            }
        }
    }

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

        guard let connectionStatus = self.tunnelProvider?.connection.status else { return }

        logger.debug("VPN status changed to \(connectionStatus)")

        let operation = MapConnectionStatusOperation(queue: stateQueue, connectionStatus: connectionStatus, delegate: self)

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

    // MARK: - StartTunnelOperationDelegate

    func operationDidRequestTunnelInfo(_ operation: Operation) -> TunnelInfo? {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        return tunnelInfo
    }

    func operationDidRequestTunnelState(_ operation: Operation) -> TunnelState {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        return tunnelState
    }

    func operation(_ operation: Operation, didSetTunnelState newTunnelState: TunnelState) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        tunnelState = newTunnelState
    }

    func operation(_ operation: Operation, didSetTunnelProvider newTunnelProvider: TunnelProviderManagerType) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        guard tunnelProvider != newTunnelProvider else {
            return
        }

        // Save the new active tunnel provider
        tunnelProvider = newTunnelProvider

        // Register for tunnel connection status changes
        subscribeVPNStatusObserver(for: newTunnelProvider)

        // Update the existing state
        updateTunnelState()
    }

    func operation(_ operation: Operation, didFailToStartTunnelWithError error: TunnelManager.Error) {
        logger.error(chainedError: error)
    }

    func operation(_ operation: Operation, didFailToEncodeTunnelOptions error: Swift.Error) {
        logger.error(chainedError: AnyChainedError(error), message: "Failed to encode tunnel options")
    }

    // MARK: - StopTunnelOperationDelegate

    func operationDidRequestTunnelProvider(_ operation: Operation) -> TunnelProviderManagerType? {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        return tunnelProvider
    }

    func operation(_ operation: Operation, didFailToStopTunnelWithError error: TunnelManager.Error) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        // Pass tunnel failure to observers
        DispatchQueue.main.async {
            self.observerList.forEach { observer in
                observer.tunnelManager(self, didFailWithError: error)
            }
        }
    }

    // MARK: - ReloadTunnelOperationDelegate

    func operation(_ operation: Operation, didFailToReloadTunnelWithError error: TunnelIPC.Error) {
        logger.error(chainedError: error, message: "Failed to send IPC request to reload tunnel settings")
    }

    // MARK: - MapConnectionStatusOperationDelegate

    func operationDidRequestTunnelToStart(_ operation: Operation) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        startTunnel()
    }

    // MARK: - RenegeratePrivateKeyOperationDelegate

    func operation(_ operation: Operation, didFinishRegeneratingPrivateKeyWithNewTunnelSettings newTunnelSettings: TunnelSettings) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        tunnelInfo?.tunnelSettings = newTunnelSettings

        updatePrivateKeyRotationTimer()
        reconnectTunnel(completionHandler: nil)
    }

    func operation(_ operation: Operation, didFailToReplacePrivateKeyWithError error: TunnelManager.Error) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        logger.error(chainedError: error, message: "Failed to regenerate private key")
    }

    // MARK: - RotatePrivateKeyOperationDelegate

    func operation(_ operation: Operation, didFinishRotatingPrivateKeyWithNewTunnelSettings newTunnelSettings: TunnelSettings) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        tunnelInfo?.tunnelSettings = newTunnelSettings
    }

    func operation(_ operation: Operation, didFailToRotatePrivateKeyWithError error: Error) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        logger.error(chainedError: error, message: "Failed to rotate private key")
    }

    // MARK: - LoadTunnelOperationDelegate

    func operation(_ operation: Operation, didSetTunnelInfo newTunnelInfo: TunnelInfo) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        self.tunnelInfo = newTunnelInfo
    }

    func operation(_ operation: Operation, didFailToLoadTunnelWithError error: TunnelManager.Error) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        logger.error(chainedError: error, message: "Failed to load tunnel")
    }

    func operationDidFinishLoadingTunnel(_ operation: Operation) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        updatePrivateKeyRotationTimer()
    }

    // MARK: - SetAccountOperationDelegate

    func operation(_ operation: Operation, didFailToSetAccountWithError error: TunnelManager.Error) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        logger.error(chainedError: error, message: "Failed to set account token")
    }

    func operationDidSetAccountToken(_ operation: Operation) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        updatePrivateKeyRotationTimer()
    }

    // MARK: - UnsetAccountOperationDelegate

    func operationWillDeleteVPNConfiguration(_ operation: Operation) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        // Unregister from receiving VPN connection status changes
        unsubscribeVPNStatusObserver()

        // Cancel last VPN status mapping operation
        lastMapConnectionStatusOperation?.cancel()
        lastMapConnectionStatusOperation = nil

        // Reset tunnel state to disconnected
        tunnelState = .disconnected

        // Remove tunnel info
        tunnelInfo = nil
    }

    func operationDidUnsetAccount(_ operation: Operation) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        tunnelProvider = nil
        updatePrivateKeyRotationTimer()
    }

    // MARK: - SetTunnelSettingsOperationDelegate

    func operation(_ operation: Operation, didFailToSetTunnelSettingsWithError error: Error) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        logger.error(chainedError: error, message: "Failed to set tunnel settings")
    }

    func operation(_ operation: Operation, didSetTunnelSettings newTunnelSettings: TunnelSettings) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        tunnelInfo?.tunnelSettings = newTunnelSettings
        reconnectTunnel(completionHandler: nil)
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
        if let tunnelInfo = self.tunnelInfo {
            let creationDate = tunnelInfo.tunnelSettings.interface.privateKey.creationDate
            let beginDate = Date(timeInterval: Self.privateKeyRotationInterval, since: creationDate)

            return submitBackgroundTask(at: beginDate)
        } else {
            return .failure(.missingAccount)
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
        case .missingAccount:
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
