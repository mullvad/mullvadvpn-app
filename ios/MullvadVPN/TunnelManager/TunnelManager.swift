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
final class TunnelManager {
    private enum OperationCategory: String {
        case manageTunnel
        case deviceStateUpdate
        case settingsUpdate
        case tunnelStateUpdate

        var category: String {
            return "TunnelManager.\(rawValue)"
        }
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
    private var nslock = NSRecursiveLock()
    private let operationQueue = AsyncOperationQueue()
    private let internalQueue = DispatchQueue(label: "TunnelManager.internalQueue")

    private var statusObserver: TunnelStatusBlockObserver?
    private var lastMapConnectionStatusOperation: Operation?
    private let observerList = ObserverList<TunnelObserver>()

    private var privateKeyRotationTimer: DispatchSourceTimer?
    private var lastKeyRotationData: (
        attempt: Date,
        completion: OperationCompletion<Bool, Error>
    )?
    private var isRunningPeriodicPrivateKeyRotation = false

    private var tunnelStatusPollTimer: DispatchSourceTimer?
    private var isPolling = false
    private var lastConnectingDate: Date?

    private var _isConfigurationLoaded = false
    private var _deviceState: DeviceState = .loggedOut
    private var _tunnelSettings = TunnelSettingsV2()

    private var _tunnel: Tunnel?
    private var _tunnelStatus = TunnelStatus()

    // MARK: - Initialization

    private init(accountsProxy: REST.AccountsProxy, devicesProxy: REST.DevicesProxy) {
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
        self.operationQueue.name = "TunnelManager.operationQueue"
        self.operationQueue.underlyingQueue = internalQueue
    }

    // MARK: - Periodic private key rotation

    func startPeriodicPrivateKeyRotation() {
        nslock.lock()
        defer { nslock.unlock() }

        guard !isRunningPeriodicPrivateKeyRotation else { return }

        logger.debug("Start periodic private key rotation.")

        isRunningPeriodicPrivateKeyRotation = true
        updatePrivateKeyRotationTimer()

        nslock.unlock()
    }

    func stopPeriodicPrivateKeyRotation() {
        nslock.lock()
        defer { nslock.unlock() }

        guard isRunningPeriodicPrivateKeyRotation else { return }

        logger.debug("Stop periodic private key rotation.")

        isRunningPeriodicPrivateKeyRotation = false
        updatePrivateKeyRotationTimer()
    }

    func getNextKeyRotationDate() -> Date? {
        nslock.lock()
        defer { nslock.unlock() }

        guard case .loggedIn(_, let deviceData) = deviceState else {
            return nil
        }

        if case .some(let (lastAttemptDate, completion)) = lastKeyRotationData {
            if completion.error is InvalidDeviceStateError {
                return nil
            }

            if completion.error is RevokedDeviceError {
                return nil
            }

            // Do not rotate the key if account or device is not found.
            if let restError = completion.error as? REST.Error,
               restError.compareErrorCode(.invalidAccount) ||
                restError.compareErrorCode(.deviceNotFound) {
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
        let date = deviceData.wgKeyData.creationDate
            .addingTimeInterval(TunnelManagerConfiguration.privateKeyRotationInterval)

        return max(date, Date())
    }

    private func updatePrivateKeyRotationTimer() {
        nslock.lock()
        defer { nslock.unlock() }

        privateKeyRotationTimer?.cancel()
        privateKeyRotationTimer = nil

        guard isRunningPeriodicPrivateKeyRotation,
              let scheduleDate = getNextKeyRotationDate() else { return }

        let timer = DispatchSource.makeTimerSource(queue: .main)

        timer.setEventHandler { [weak self] in
            _ = self?.rotatePrivateKey(forceRotate: false) { _ in
                // no-op
            }
        }

        timer.schedule(wallDeadline: .now() + scheduleDate.timeIntervalSinceNow)
        timer.activate()

        privateKeyRotationTimer = timer

        logger.debug("Schedule next private key rotation at \(scheduleDate.logFormatDate()).")
    }

    private func setFinishedKeyRotation(_ completion: OperationCompletion<Bool, Error>) {
        nslock.lock()
        defer { nslock.unlock() }

        lastKeyRotationData = (Date(), completion)
        updatePrivateKeyRotationTimer()
    }

    private func resetKeyRotationData() {
        nslock.lock()
        defer { nslock.unlock() }

        lastKeyRotationData = nil
        updatePrivateKeyRotationTimer()

    }

    // MARK: - Public methods

    func loadConfiguration(completionHandler: @escaping (Error?) -> Void) {
        let migrateSettingsOperation = MigrateSettingsOperation(
            dispatchQueue: internalQueue,
            accountsProxy: accountsProxy,
            devicesProxy: devicesProxy
        )

        let loadTunnelOperation = LoadTunnelConfigurationOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self)
        )
        loadTunnelOperation.completionQueue = .main
        loadTunnelOperation.completionHandler = { [weak self] completion in
            guard let self = self else { return }

            if case .failure(let error) = completion {
                self.logger.error(
                    chainedError: AnyChainedError(error),
                    message: "Failed to load configuration."
                )
            }

            self.updatePrivateKeyRotationTimer()

            completionHandler(completion.error)
        }
        loadTunnelOperation.addDependency(migrateSettingsOperation)

        let groupOperation = GroupOperation(operations: [
            migrateSettingsOperation, loadTunnelOperation
        ])

        groupOperation.addObserver(
            BackgroundObserver(name: "Load tunnel configuration", cancelUponExpiration: false)
        )

        groupOperation.addCondition(
            MutuallyExclusive(category: OperationCategory.manageTunnel.category)
        )
        groupOperation.addCondition(
            MutuallyExclusive(category: OperationCategory.deviceStateUpdate.category)
        )
        groupOperation.addCondition(
            MutuallyExclusive(category: OperationCategory.settingsUpdate.category)
        )

        operationQueue.addOperation(groupOperation)
    }

    func refreshTunnelStatus() {
        logger.debug("Refresh tunnel status due to application becoming active.")
        _refreshTunnelStatus()
    }

    func startTunnel() {
        let operation = StartTunnelOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self),
            completionHandler: { [weak self] completion in
                guard let self = self, let error = completion.error else { return }

                self.logger.error(
                    chainedError: AnyChainedError(error),
                    message: "Failed to start the tunnel."
                )

                DispatchQueue.main.async {
                    let tunnelError = StartTunnelError(underlyingError: error)

                    self.observerList.forEach { observer in
                        observer.tunnelManager(self, didFailWithError: tunnelError)
                    }
                }
            })

        operation.addObserver(BackgroundObserver(name: "Start tunnel", cancelUponExpiration: true))
        operation.addCondition(MutuallyExclusive(category: OperationCategory.manageTunnel.category))

        operationQueue.addOperation(operation)
    }

    func stopTunnel() {
        let operation = StopTunnelOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self)
        ) { [weak self] completion in
            guard let self = self, let error = completion.error else { return }

            self.logger.error(
                chainedError: AnyChainedError(error),
                message: "Failed to stop the tunnel."
            )

            DispatchQueue.main.async {
                let tunnelError = StopTunnelError(underlyingError: error)

                self.observerList.forEach { observer in
                    observer.tunnelManager(self, didFailWithError: tunnelError)
                }
            }
        }

        operation.addObserver(BackgroundObserver(name: "Stop tunnel", cancelUponExpiration: true))
        operation.addCondition(MutuallyExclusive(category: OperationCategory.manageTunnel.category))

        operationQueue.addOperation(operation)
    }

    func reconnectTunnel(
        selectNewRelay: Bool,
        completionHandler: ((OperationCompletion<(), Error>) -> Void)? = nil
    )
    {
        let operation = ReconnectTunnelOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self),
            selectNewRelay: selectNewRelay
        )

        operation.completionQueue = .main
        operation.completionHandler = { [weak self] completion in
            self?.didReconnectTunnel(completion: completion)

            completionHandler?(completion)
        }

        operation.addObserver(
            BackgroundObserver(name: "Reconnect tunnel", cancelUponExpiration: true)
        )
        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.manageTunnel.category)
        )

        operationQueue.addOperation(operation)
    }

    func setAccount(action: SetAccountAction, completionHandler: @escaping (OperationCompletion<StoredAccountData?, Error>) -> Void) {
        let operation = SetAccountOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self),
            accountsProxy: accountsProxy,
            devicesProxy: devicesProxy,
            action: action
        )

        operation.completionQueue = .main
        operation.completionHandler = { [weak self] completion in
            self?.resetKeyRotationData()

            completionHandler(completion)
        }

        operation.addObserver(BackgroundObserver(name: action.taskName, cancelUponExpiration: true))

        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.manageTunnel.category)
        )
        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.deviceStateUpdate.category)
        )
        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.settingsUpdate.category)
        )

        operationQueue.addOperation(operation)
    }

    func unsetAccount(completionHandler: @escaping () -> Void) {
        setAccount(action: .unset) { _ in
            completionHandler()
        }
    }

    func updateAccountData(_ completionHandler: ((Error?) -> Void)? = nil) {
        let operation = UpdateAccountDataOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self),
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
            MutuallyExclusive(category: OperationCategory.deviceStateUpdate.category)
        )

        operationQueue.addOperation(operation)
    }

    func updateDeviceData(_ completionHandler: @escaping (OperationCompletion<StoredDeviceData, Error>) -> Void) -> Cancellable {
        let operation = UpdateDeviceDataOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self),
            devicesProxy: devicesProxy
        )

        operation.completionQueue = .main
        operation.completionHandler = { [weak self] completion in
            if completion.error is RevokedDeviceError {
                self?.didDetectDeviceRevoked()
            }
            completionHandler(completion)
        }

        operation.addObserver(
            BackgroundObserver(name: "Update device data", cancelUponExpiration: true)
        )

        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.deviceStateUpdate.category)
        )

        operationQueue.addOperation(operation)

        return operation
    }

    func rotatePrivateKey(
        forceRotate: Bool,
        completionHandler: @escaping (OperationCompletion<Bool, Error>) -> Void
    ) -> Cancellable {
        var rotationInterval: TimeInterval?
        if !forceRotate {
            rotationInterval = TunnelManagerConfiguration.privateKeyRotationInterval
        }

        let operation = RotateKeyOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self),
            devicesProxy: devicesProxy,
            rotationInterval: rotationInterval
        )

        operation.completionQueue = .main
        operation.completionHandler = { [weak self] completion in
            guard let self = self else { return }

            self.setFinishedKeyRotation(completion)

            switch completion {
            case .success:
                self.reconnectTunnel(selectNewRelay: true) { _ in
                    completionHandler(completion)
                }

            case .failure(let error):
                self.logger.error(
                    chainedError: AnyChainedError(error),
                    message: "Failed to rotate private key."
                )

                completionHandler(completion)

            case .cancelled:
                completionHandler(completion)
            }
        }

        operation.addObserver(
            BackgroundObserver(name: "Rotate private key", cancelUponExpiration: true)
        )

        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.deviceStateUpdate.category)
        )

        operationQueue.addOperation(operation)

        return operation
    }

    func setRelayConstraints(_ newConstraints: RelayConstraints, completionHandler: (() -> Void)? = nil) {
        scheduleSettingsUpdate(
            taskName: "Set relay constraints",
            modificationBlock: { settings in
                settings.relayConstraints = newConstraints
            },
            completionHandler: completionHandler
        )
    }

    func setDNSSettings(_ newDNSSettings: DNSSettings, completionHandler: (() -> Void)? = nil) {
        scheduleSettingsUpdate(
            taskName: "Set DNS settings",
            modificationBlock: { settings in
                settings.dnsSettings = newDNSSettings
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

    // MARK: - TunnelInteractor

    var isConfigurationLoaded: Bool {
        nslock.lock()
        defer { nslock.unlock() }

        return _isConfigurationLoaded
    }

    fileprivate var tunnel: Tunnel? {
        nslock.lock()
        defer { nslock.unlock() }

        return _tunnel
    }

    var tunnelStatus: TunnelStatus {
        nslock.lock()
        defer { nslock.unlock() }

        return _tunnelStatus
    }

    var settings: TunnelSettingsV2 {
        nslock.lock()
        defer { nslock.unlock() }

        return _tunnelSettings
    }

    var deviceState: DeviceState {
        nslock.lock()
        defer { nslock.unlock() }

        return _deviceState
    }

    fileprivate func setConfigurationLoaded() {
        nslock.lock()
        defer { nslock.unlock() }

        guard !_isConfigurationLoaded else {
            return
        }

        _isConfigurationLoaded = true

        DispatchQueue.main.async {
            self.observerList.forEach { observer in
                observer.tunnelManagerDidLoadConfiguration(self)
            }
        }
    }

    fileprivate func setTunnel(_ tunnel: Tunnel?, shouldRefreshTunnelState: Bool) {
        nslock.lock()
        defer { nslock.unlock() }

        if let tunnel = tunnel {
            subscribeVPNStatusObserver(tunnel: tunnel)
        } else {
            unsubscribeVPNStatusObserver()
        }

        _tunnel = tunnel

        // Update the existing state
        if shouldRefreshTunnelState {
            logger.debug("Refresh tunnel status for new tunnel.")
            _refreshTunnelStatus()
        }
    }

    fileprivate func updateTunnelState(_ state: TunnelState) {
        nslock.lock()
        defer { nslock.unlock() }

        var updatedStatus = _tunnelStatus
        updatedStatus.state = state
        setTunnelStatus(updatedStatus)
    }

    fileprivate func updateTunnelStatus(
        from packetTunnelStatus: PacketTunnelStatus,
        mappingRelayToState mapper: (PacketTunnelRelay?) -> TunnelState?
    )
    {
        nslock.lock()
        defer { nslock.unlock() }

        var updatedStatus = _tunnelStatus
        updatedStatus.update(from: packetTunnelStatus, mappingRelayToState: mapper)
        setTunnelStatus(updatedStatus)
    }

    fileprivate func resetTunnelStatus(to state: TunnelState) {
        nslock.lock()
        defer { nslock.unlock() }

        var updatedStatus = _tunnelStatus
        updatedStatus.reset(to: state)
        setTunnelStatus(updatedStatus)
    }

    fileprivate func setTunnelStatus(_ newTunnelStatus: TunnelStatus) {
        nslock.lock()
        defer { nslock.unlock() }

        logger.info("Status: \(newTunnelStatus).")

        _tunnelStatus = newTunnelStatus

        switch newTunnelStatus.state {
        case .connecting, .reconnecting:
            // Start polling tunnel status to keep the relay information up to date
            // while the tunnel process is trying to connect.
            startPollingTunnelStatus(
                connectingDate: newTunnelStatus.packetTunnelStatus.connectingDate
            )

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

    fileprivate func setSettings(_ settings: TunnelSettingsV2, persist: Bool) {
        nslock.lock()
        defer { nslock.unlock() }

        let shouldCallDelegate = _tunnelSettings != settings && _isConfigurationLoaded

        _tunnelSettings = settings

        if persist {
            do {
                try SettingsManager.writeSettings(settings)
            } catch {
                logger.error(
                    chainedError: AnyChainedError(error),
                    message: "Failed to write settings."
                )
            }
        }

        if shouldCallDelegate {
            DispatchQueue.main.async {
                self.observerList.forEach { observer in
                    observer.tunnelManager(self, didUpdateTunnelSettings: settings)
                }
            }
        }
    }

    fileprivate func setDeviceState(_ deviceState: DeviceState, persist: Bool) {
        nslock.lock()
        defer { nslock.unlock() }

        let shouldCallDelegate = _deviceState != deviceState && _isConfigurationLoaded

        _deviceState = deviceState

        if persist {
            do {
                try SettingsManager.writeDeviceState(deviceState)
            } catch {
                logger.error(
                    chainedError: AnyChainedError(error),
                    message: "Failed to write device state."
                )
            }
        }

        if shouldCallDelegate {
            DispatchQueue.main.async {
                self.observerList.forEach { observer in
                    observer.tunnelManager(self, didUpdateDeviceState: deviceState)
                }
            }
        }
    }

    // MARK: - Private methods

    fileprivate func prepareForVPNConfigurationDeletion() {
        nslock.lock()
        defer { nslock.unlock() }

        // Unregister from receiving VPN connection status changes
        unsubscribeVPNStatusObserver()

        // Cancel last VPN status mapping operation
        lastMapConnectionStatusOperation?.cancel()
        lastMapConnectionStatusOperation = nil
    }

    private func didDetectDeviceRevoked() {
        scheduleDeviceStateUpdate(
            taskName: "Set device revoked",
            modificationBlock: { deviceState in
                deviceState = .revoked
            },
            completionHandler: nil
        )
    }

    private func didReconnectTunnel(completion: OperationCompletion<(), Error>) {
        nslock.lock()
        defer { nslock.unlock() }

        if let error = completion.error {
            logger.error(
                chainedError: AnyChainedError(error),
                message: "Failed to reconnect the tunnel."
            )
        }

        // Refresh tunnel status only when connecting or reasserting to pick up the next relay,
        // since both states may persist for a long period of time until the tunnel is fully
        // connected.
        switch tunnelStatus.state {
        case .connecting, .reconnecting:
            logger.debug("Refresh tunnel status due to reconnect.")
            _refreshTunnelStatus()

        default:
            break
        }
    }

    private func subscribeVPNStatusObserver(tunnel: Tunnel) {
        nslock.lock()
        defer { nslock.unlock() }

        unsubscribeVPNStatusObserver()

        statusObserver = tunnel.addBlockObserver(queue: internalQueue) { [weak self] tunnel, status in
            guard let self = self else { return }

            self.logger.debug("VPN connection status changed to \(status).")
            self.updateTunnelStatus(status)
        }
    }

    private func unsubscribeVPNStatusObserver() {
        nslock.lock()
        defer { nslock.unlock() }

        statusObserver?.invalidate()
        statusObserver = nil
    }

    private func _refreshTunnelStatus() {
        nslock.lock()
        defer { nslock.unlock() }

        if let connectionStatus = _tunnel?.status {
            updateTunnelStatus(connectionStatus)
        }
    }

    /// Update `TunnelStatus` from `NEVPNStatus`.
    /// Collects the `PacketTunnelStatus` from the tunnel via IPC if needed before assigning
    /// the `tunnelStatus`.
    private func updateTunnelStatus(_ connectionStatus: NEVPNStatus) {
        nslock.lock()
        defer { nslock.unlock() }

        let operation = MapConnectionStatusOperation(
            queue: internalQueue,
            interactor: TunnelInteractorProxy(self),
            connectionStatus: connectionStatus
        )

        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.tunnelStateUpdate.category)
        )

        // Cancel last VPN status mapping operation
        lastMapConnectionStatusOperation?.cancel()
        lastMapConnectionStatusOperation = operation

        operationQueue.addOperation(operation)
    }

    private func scheduleSettingsUpdate(
        taskName: String,
        modificationBlock: @escaping (inout TunnelSettingsV2) -> Void,
        completionHandler: (() -> Void)?
    )
    {
        let operation = AsyncBlockOperation(dispatchQueue: internalQueue) {
            let currentSettings = self._tunnelSettings
            var updatedSettings = self._tunnelSettings

            modificationBlock(&updatedSettings)

            // Select new relay only when relay constraints change.
            let currentConstraints = currentSettings.relayConstraints
            let updatedConstraints = updatedSettings.relayConstraints
            let selectNewRelay = currentConstraints != updatedConstraints

            self.setSettings(updatedSettings, persist: true)
            self.reconnectTunnel(selectNewRelay: selectNewRelay, completionHandler: nil)
        }

        operation.completionBlock = {
            DispatchQueue.main.async {
                completionHandler?()
            }
        }

        operation.addObserver(BackgroundObserver(name: taskName, cancelUponExpiration: false))
        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.settingsUpdate.category)
        )

        operationQueue.addOperation(operation)
    }

    private func scheduleDeviceStateUpdate(
        taskName: String,
        modificationBlock: @escaping (inout DeviceState) -> Void,
        completionHandler: (() -> Void)?
    )
    {
        let operation = AsyncBlockOperation(dispatchQueue: internalQueue) {
            var deviceState = self.deviceState

            modificationBlock(&deviceState)

            self.setDeviceState(deviceState, persist: true)
            self.reconnectTunnel(selectNewRelay: false, completionHandler: nil)
        }

        operation.completionBlock = {
            DispatchQueue.main.async {
                completionHandler?()
            }
        }

        operation.addObserver(BackgroundObserver(name: taskName, cancelUponExpiration: false))
        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.deviceStateUpdate.category)
        )

        operationQueue.addOperation(operation)
    }

    // MARK: - Tunnel status polling

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

        let timer = DispatchSource.makeTimerSource(queue: .main)
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
    func appStorePaymentManager(
        _ manager: AppStorePaymentManager,
        transaction: SKPaymentTransaction?,
        payment: SKPayment,
        accountToken: String?,
        didFailWithError error: AppStorePaymentManager.Error
    )
    {
        // no-op
    }

    func appStorePaymentManager(
        _ manager: AppStorePaymentManager,
        transaction: SKPaymentTransaction,
        accountToken: String,
        didFinishWithResponse response: REST.CreateApplePaymentResponse
    )
    {
        scheduleDeviceStateUpdate(
            taskName: "Update account expiry after in-app purchase",
            modificationBlock: { deviceState in
                switch deviceState {
                case .loggedIn(var accountData, let deviceData):
                    if accountData.number == accountToken {
                        accountData.expiry = response.newExpiry
                        deviceState = .loggedIn(accountData, deviceData)
                    }

                case .loggedOut, .revoked:
                    break
                }
            },
            completionHandler: nil
        )
    }
}

private struct TunnelInteractorProxy: TunnelInteractor {
    private let tunnelManager: TunnelManager

    init(_ tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
    }

    var tunnel: Tunnel? {
        return tunnelManager.tunnel
    }

    func setTunnel(_ tunnel: Tunnel?, shouldRefreshTunnelState: Bool) {
        tunnelManager.setTunnel(tunnel, shouldRefreshTunnelState: shouldRefreshTunnelState)
    }

    var tunnelStatus: TunnelStatus {
        return tunnelManager.tunnelStatus
    }

    func setTunnelStatus(_ tunnelStatus: TunnelStatus) {
        tunnelManager.setTunnelStatus(tunnelStatus)
    }

    func updateTunnelStatus(
        from packetTunnelStatus: PacketTunnelStatus,
        mappingRelayToState mapper: (PacketTunnelRelay?) -> TunnelState?
    )
    {
        tunnelManager.updateTunnelStatus(from: packetTunnelStatus, mappingRelayToState: mapper)
    }

    func updateTunnelState(_ state: TunnelState) {
        tunnelManager.updateTunnelState(state)
    }

    func resetTunnelState(to state: TunnelState) {
        tunnelManager.resetTunnelStatus(to: state)
    }

    var isConfigurationLoaded: Bool {
        return tunnelManager.isConfigurationLoaded
    }

    var settings: TunnelSettingsV2 {
        return tunnelManager.settings
    }

    var deviceState: DeviceState {
        return tunnelManager.deviceState
    }

    func setConfigurationLoaded() {
        tunnelManager.setConfigurationLoaded()
    }

    func setSettings(_ settings: TunnelSettingsV2, persist: Bool) {
        tunnelManager.setSettings(settings, persist: persist)
    }

    func setDeviceState(_ deviceState: DeviceState, persist: Bool) {
        tunnelManager.setDeviceState(deviceState, persist: persist)
    }

    func startTunnel() {
        tunnelManager.startTunnel()
    }

    func prepareForVPNConfigurationDeletion() {
        tunnelManager.prepareForVPNConfigurationDeletion()
    }

}
