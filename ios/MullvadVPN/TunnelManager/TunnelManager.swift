//
//  TunnelManager.swift
//  MullvadVPN
//
//  Created by pronebird on 25/09/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

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
    private let operationQueue = AsyncOperationQueue()

    private var statusObserver: TunnelStatusBlockObserver?
    private var lastMapConnectionStatusOperation: Operation?
    private let observerList = ObserverList<TunnelObserver>()

    private let state: TunnelManager.State

    private var privateKeyRotationTimer: DispatchSourceTimer?
    private var lastKeyRotationData: (
        attempt: Date,
        completion: OperationCompletion<Bool, TunnelManager.Error>
    )?
    private var isRunningPeriodicPrivateKeyRotation = false

    private var tunnelStatusPollTimer: DispatchSourceTimer?
    private var isPolling = false
    private var lastConnectingDate: Date?

    var isLoadedConfiguration: Bool {
        return state.isLoadedConfiguration
    }

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
        self.state = TunnelManager.State(delegateQueue: stateQueue)
        self.state.delegate = self
        self.operationQueue.name = "TunnelManager.operationQueue"
        self.operationQueue.underlyingQueue = stateQueue
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
            self.updatePrivateKeyRotationTimer()
        }
    }

    func getNextKeyRotationDate() -> Date? {
        return stateQueue.sync {
            return _getNextKeyRotationDate()
        }
    }

    private func updatePrivateKeyRotationTimer() {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        privateKeyRotationTimer?.cancel()
        privateKeyRotationTimer = nil

        guard self.isRunningPeriodicPrivateKeyRotation else { return }

        guard let scheduleDate = _getNextKeyRotationDate() else { return }

        let timer = DispatchSource.makeTimerSource(queue: stateQueue)

        timer.setEventHandler { [weak self] in
            guard let self = self else { return }

            _ = self.rotatePrivateKey(forceRotate: false) { _ in
                // no-op
            }
        }

        timer.schedule(wallDeadline: .now() + scheduleDate.timeIntervalSinceNow)
        timer.activate()

        privateKeyRotationTimer = timer

        logger.debug("Schedule next private key rotation at \(scheduleDate.logFormatDate()).")
    }

    private func _getNextKeyRotationDate() -> Date? {
        guard let tunnelSettings = state.tunnelSettings else {
            return nil
        }

        if case .some(let (lastAttemptDate, completion)) = lastKeyRotationData {
            // Do not rotate the key when logged out.
            if case .unsetAccount = completion.error {
                return nil
            }

            // Do not rotate the key if account or device is not found.
            if case .rotateKey(.unhandledResponse(_, let serverErrorResponse)) = completion.error,
               serverErrorResponse?.code == .invalidAccount ||
                serverErrorResponse?.code == .deviceNotFound {
                return nil
            }

            // Retry at equal interval if failed or cancelled.
            if !completion.isSuccess {
                let date = lastAttemptDate.addingTimeInterval(
                    TunnelManagerConfiguration.privateKeyRotationFailureRetryInterval
                )

                return max(date, Date())
            }
        }

        // Rotate at long intervals otherwise.
        let date = tunnelSettings.device.wgKeyData.creationDate
            .addingTimeInterval(TunnelManagerConfiguration.privateKeyRotationInterval)

        return max(date, Date())
    }

    private func setFinishedKeyRotation(_ completion: OperationCompletion<Bool, TunnelManager.Error>) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        lastKeyRotationData = (Date(), completion)
        updatePrivateKeyRotationTimer()
    }

    private func resetKeyRotationData() {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        lastKeyRotationData = nil
        updatePrivateKeyRotationTimer()

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
        )
        loadTunnelOperation.completionQueue = stateQueue
        loadTunnelOperation.completionHandler = { [weak self] completion in
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
        loadTunnelOperation.addDependency(migrateSettingsOperation)

        let groupOperation = GroupOperation(operations: [
            migrateSettingsOperation, loadTunnelOperation
        ])

        groupOperation.addObserver(
            BackgroundObserver(name: "Load tunnel configuration", cancelUponExpiration: false)
        )

        groupOperation.addCondition(
            MutuallyExclusive(category: OperationCategory.manageTunnelProvider)
        )
        groupOperation.addCondition(
            MutuallyExclusive(category: OperationCategory.changeTunnelSettings)
        )

        operationQueue.addOperation(groupOperation)
    }

    func refreshTunnelStatus() {
        stateQueue.async {
            self.logger.debug("Refresh tunnel status due to application becoming active.")
            self._refreshTunnelStatus()
        }
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

        operation.addObserver(BackgroundObserver(name: "Start tunnel", cancelUponExpiration: true))
        operation.addCondition(MutuallyExclusive(category: OperationCategory.manageTunnelProvider))

        operationQueue.addOperation(operation)
    }

    func stopTunnel() {
        let operation = StopTunnelOperation(
            dispatchQueue: stateQueue,
            state: state
        ) { [weak self] completion in
            guard let self = self, let error = completion.error else { return }

            // Pass tunnel failure to observers
            DispatchQueue.main.async {
                self.observerList.forEach { observer in
                    observer.tunnelManager(self, didFailWithError: error)
                }
            }
        }

        operation.addObserver(BackgroundObserver(name: "Stop tunnel", cancelUponExpiration: true))
        operation.addCondition(MutuallyExclusive(category: OperationCategory.manageTunnelProvider))

        operationQueue.addOperation(operation)
    }

    func reconnectTunnel(
        selectNewRelay: Bool,
        completionHandler: ((OperationCompletion<(), TunnelManager.Error>) -> Void)? = nil
    )
    {
        let operation = ReloadTunnelOperation(
            dispatchQueue: stateQueue,
            state: state,
            selectNewRelay: selectNewRelay
        )

        operation.completionHandler = { [weak self] completion in
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
                self._refreshTunnelStatus()

            default:
                break
            }

            DispatchQueue.main.async {
                completionHandler?(completion)
            }
        }
        operation.completionQueue = stateQueue

        operation.addObserver(
            BackgroundObserver(name: "Reconnect tunnel", cancelUponExpiration: true)
        )
        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.manageTunnelProvider)
        )

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
            })

        operation.completionQueue = stateQueue
        operation.completionHandler = { [weak self] completion in
            guard let self = self else { return }

            self.resetKeyRotationData()

            DispatchQueue.main.async {
                completionHandler(completion)
            }
        }

        operation.addObserver(BackgroundObserver(name: action.taskName, cancelUponExpiration: true))

        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.manageTunnelProvider)
        )
        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.changeTunnelSettings)
        )

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

        operation.addObserver(
            BackgroundObserver(name: "Update account data", cancelUponExpiration: true)
        )

        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.changeTunnelSettings)
        )

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

        operation.addObserver(
            BackgroundObserver(name: "Update device data", cancelUponExpiration: true)
        )

        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.changeTunnelSettings)
        )

        operationQueue.addOperation(operation)

        return operation
    }

    func rotatePrivateKey(
        forceRotate: Bool,
        completionHandler: @escaping (OperationCompletion<Bool, TunnelManager.Error>) -> Void
    ) -> Cancellable {
        var rotationInterval: TimeInterval?
        if !forceRotate {
            rotationInterval = TunnelManagerConfiguration.privateKeyRotationInterval
        }

        let operation = RotateKeyOperation(
            dispatchQueue: stateQueue,
            state: state,
            devicesProxy: devicesProxy,
            rotationInterval: rotationInterval
        ) { [weak self] completion in
            guard let self = self else { return }

            dispatchPrecondition(condition: .onQueue(self.stateQueue))
            self.setFinishedKeyRotation(completion)

            switch completion {
            case .success:
                self.reconnectTunnel(selectNewRelay: true) { _ in
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

        operation.addObserver(
            BackgroundObserver(name: "Rotate private key", cancelUponExpiration: true)
        )

        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.changeTunnelSettings)
        )

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

    func tunnelManagerState(
        _ state: State,
        didChangeLoadedConfiguration isLoadedConfiguration: Bool
    )
    {
        DispatchQueue.main.async {
            self.observerList.forEach { observer in
                if isLoadedConfiguration {
                    observer.tunnelManagerDidLoadConfiguration(self)
                }
            }
        }
    }

    func tunnelManagerState(
        _ state: TunnelManager.State,
        didChangeTunnelSettings newTunnelSettings: TunnelSettingsV2?
    )
    {
        DispatchQueue.main.async {
            self.observerList.forEach { observer in
                observer.tunnelManager(self, didUpdateTunnelSettings: newTunnelSettings)
            }
        }
    }

    func tunnelManagerState(
        _ state: TunnelManager.State,
        didChangeTunnelStatus newTunnelStatus: TunnelStatus
    )
    {
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

    func tunnelManagerState(
        _ state: TunnelManager.State,
        didChangeTunnelProvider newTunnelObject: Tunnel?,
        shouldRefreshTunnelState: Bool
    )
    {
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
            _refreshTunnelStatus()
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

    private func _refreshTunnelStatus() {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        if let connectionStatus = state.tunnel?.status {
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

        operation.addCondition(MutuallyExclusive(category: OperationCategory.tunnelStateUpdate))

        // Cancel last VPN status mapping operation
        lastMapConnectionStatusOperation?.cancel()
        lastMapConnectionStatusOperation = operation

        operationQueue.addOperation(operation)
    }

    fileprivate func scheduleTunnelSettingsUpdate(taskName: String, modificationBlock: @escaping (inout TunnelSettingsV2) -> Void, completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        let operation = ResultBlockOperation<Void, TunnelManager.Error>(
            dispatchQueue: stateQueue
        ) { operation in
            guard let currentSettings = self.tunnelSettings else {
                operation.finish(completion: .failure(.unsetAccount))
                return
            }

            do {
                var updatedSettings = currentSettings

                modificationBlock(&updatedSettings)

                // Select new relay only when relay constraints change.
                let currentConstraints = currentSettings.relayConstraints
                let updatedConstraints = updatedSettings.relayConstraints
                let selectNewRelay = currentConstraints != updatedConstraints

                try SettingsManager.writeSettings(updatedSettings)

                self.state.tunnelSettings = updatedSettings
                self.reconnectTunnel(selectNewRelay: selectNewRelay, completionHandler: nil)

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

        operation.addObserver(BackgroundObserver(name: taskName, cancelUponExpiration: true))
        operation.addCondition(MutuallyExclusive(category: OperationCategory.changeTunnelSettings))

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
            self._refreshTunnelStatus()
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

// MARK: - AppStore payment observer

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
