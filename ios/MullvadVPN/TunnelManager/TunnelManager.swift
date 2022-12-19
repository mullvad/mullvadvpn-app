//
//  TunnelManager.swift
//  MullvadVPN
//
//  Created by pronebird on 25/09/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import NetworkExtension
import Operations
import RelayCache
import RelaySelector
import StoreKit
import TunnelProviderMessaging
import UIKit
import class WireGuardKitTypes.PublicKey

/// Interval used for periodic polling of tunnel relay status when tunnel is establishing
/// connection.
private let establishingTunnelStatusPollInterval: TimeInterval = 3

/// Interval used for periodic polling of tunnel connectivity status once the tunnel connection
/// is established.
private let establishedTunnelStatusPollInterval: TimeInterval = 5

/// Private key rotation interval (in seconds).
private let privateKeyRotationInterval: TimeInterval = 60 * 60 * 24 * 4

/// Private key rotation retry interval (in seconds).
private let privateKeyRotationFailureRetryInterval: TimeInterval = 60 * 15

/// A class that provides a convenient interface for VPN tunnels configuration, manipulation and
/// monitoring.
final class TunnelManager: StorePaymentObserver {
    private enum OperationCategory: String {
        case manageTunnel
        case deviceStateUpdate
        case settingsUpdate
        case tunnelStateUpdate

        var category: String {
            return "TunnelManager.\(rawValue)"
        }
    }

    // MARK: - Internal variables

    private let application: UIApplication
    fileprivate let tunnelStore: TunnelStore
    private let relayCacheTracker: RelayCacheTracker
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

    private var _isConfigurationLoaded = false
    private var _deviceState: DeviceState = .loggedOut
    private var _tunnelSettings = TunnelSettingsV2()

    private var _tunnel: Tunnel?
    private var _tunnelStatus = TunnelStatus()

    /// Last processed device check identifier.
    private var lastDeviceCheckIdentifier: UUID?

    // MARK: - Initialization

    init(
        application: UIApplication,
        tunnelStore: TunnelStore,
        relayCacheTracker: RelayCacheTracker,
        accountsProxy: REST.AccountsProxy,
        devicesProxy: REST.DevicesProxy
    ) {
        self.application = application
        self.tunnelStore = tunnelStore
        self.relayCacheTracker = relayCacheTracker
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
        self.operationQueue.name = "TunnelManager.operationQueue"
        self.operationQueue.underlyingQueue = internalQueue

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(applicationDidBecomeActive(_:)),
            name: UIApplication.didBecomeActiveNotification,
            object: application
        )
    }

    // MARK: - Periodic private key rotation

    func startPeriodicPrivateKeyRotation() {
        nslock.lock()
        defer { nslock.unlock() }

        guard !isRunningPeriodicPrivateKeyRotation else { return }

        logger.debug("Start periodic private key rotation.")

        isRunningPeriodicPrivateKeyRotation = true
        updatePrivateKeyRotationTimer()
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

        guard case let .loggedIn(_, deviceData) = deviceState else {
            return nil
        }

        if case .some(let (lastAttemptDate, completion)) = lastKeyRotationData {
            if completion.error is InvalidDeviceStateError {
                return nil
            }

            // Do not rotate the key if account or device is not found.
            if let restError = completion.error as? REST.Error,
               restError.compareErrorCode(.invalidAccount) ||
               restError.compareErrorCode(.deviceNotFound)
            {
                return nil
            }

            // Retry at equal interval if failed or cancelled.
            if !completion.isSuccess {
                let date = lastAttemptDate.addingTimeInterval(
                    privateKeyRotationFailureRetryInterval
                )

                return max(date, Date())
            }
        }

        // Rotate at long intervals otherwise.
        let date = deviceData.wgKeyData.creationDate.addingTimeInterval(privateKeyRotationInterval)

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
        let loadTunnelOperation = LoadTunnelConfigurationOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self)
        )
        loadTunnelOperation.completionQueue = .main
        loadTunnelOperation.completionHandler = { [weak self] completion in
            guard let self = self else { return }

            if case let .failure(error) = completion {
                self.logger.error(
                    error: error,
                    message: "Failed to load configuration."
                )
            }

            self.updatePrivateKeyRotationTimer()

            completionHandler(completion.error)
        }

        loadTunnelOperation.addObserver(
            BackgroundObserver(
                application: application,
                name: "Load tunnel configuration",
                cancelUponExpiration: false
            )
        )

        loadTunnelOperation.addCondition(
            MutuallyExclusive(category: OperationCategory.manageTunnel.category)
        )

        operationQueue.addOperation(loadTunnelOperation)
    }

    func startTunnel(completionHandler: ((OperationCompletion<Void, Error>) -> Void)? = nil) {
        let operation = StartTunnelOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self),
            completionHandler: { [weak self] completion in
                guard let self = self else { return }

                DispatchQueue.main.async {
                    if let error = completion.error {
                        self.logger.error(
                            error: error,
                            message: "Failed to start the tunnel."
                        )

                        let tunnelError = StartTunnelError(underlyingError: error)

                        self.observerList.forEach { observer in
                            observer.tunnelManager(self, didFailWithError: tunnelError)
                        }
                    }

                    completionHandler?(completion)
                }
            }
        )

        operation.addObserver(BackgroundObserver(
            application: application,
            name: "Start tunnel",
            cancelUponExpiration: true
        ))
        operation.addCondition(MutuallyExclusive(category: OperationCategory.manageTunnel.category))

        operationQueue.addOperation(operation)
    }

    func stopTunnel(completionHandler: ((OperationCompletion<Void, Error>) -> Void)? = nil) {
        let operation = StopTunnelOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self)
        ) { [weak self] completion in
            guard let self = self else { return }

            DispatchQueue.main.async {
                if let error = completion.error {
                    self.logger.error(
                        error: error,
                        message: "Failed to stop the tunnel."
                    )

                    let tunnelError = StopTunnelError(underlyingError: error)

                    self.observerList.forEach { observer in
                        observer.tunnelManager(self, didFailWithError: tunnelError)
                    }
                }

                completionHandler?(completion)
            }
        }

        operation.addObserver(BackgroundObserver(
            application: application,
            name: "Stop tunnel",
            cancelUponExpiration: true
        ))
        operation.addCondition(MutuallyExclusive(category: OperationCategory.manageTunnel.category))

        operationQueue.addOperation(operation)
    }

    func reconnectTunnel(
        selectNewRelay: Bool,
        completionHandler: ((OperationCompletion<Void, Error>) -> Void)? = nil
    ) {
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
            BackgroundObserver(
                application: application,
                name: "Reconnect tunnel",
                cancelUponExpiration: true
            )
        )
        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.manageTunnel.category)
        )

        operationQueue.addOperation(operation)
    }

    func setAccount(
        action: SetAccountAction,
        completionHandler: @escaping (OperationCompletion<StoredAccountData?, Error>) -> Void
    ) {
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

        operation.addObserver(BackgroundObserver(
            application: application,
            name: action.taskName,
            cancelUponExpiration: true
        ))

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
            BackgroundObserver(
                application: application,
                name: "Update account data",
                cancelUponExpiration: true
            )
        )

        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.deviceStateUpdate.category)
        )

        operationQueue.addOperation(operation)
    }

    func updateDeviceData(_ completionHandler: ((Error?) -> Void)? = nil) {
        let operation = UpdateDeviceDataOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self),
            devicesProxy: devicesProxy
        )

        operation.completionQueue = .main
        operation.completionHandler = { [weak self] completion in
            guard let self = self else { return }

            let error = completion.error
            if let error = error {
                self.checkIfDeviceRevoked(error)
            }

            completionHandler?(error)
        }

        operation.addObserver(
            BackgroundObserver(
                application: application,
                name: "Update device data",
                cancelUponExpiration: true
            )
        )

        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.deviceStateUpdate.category)
        )

        operationQueue.addOperation(operation)
    }

    func rotatePrivateKey(
        forceRotate: Bool,
        completionHandler: @escaping (OperationCompletion<Bool, Error>) -> Void
    ) -> Cancellable {
        var rotationInterval: TimeInterval?
        if !forceRotate {
            rotationInterval = privateKeyRotationInterval
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

            case let .failure(error):
                self.checkIfDeviceRevoked(error)

                completionHandler(.failure(error))

            case .cancelled:
                completionHandler(completion)
            }
        }

        operation.addObserver(
            BackgroundObserver(
                application: application,
                name: "Rotate private key",
                cancelUponExpiration: true
            )
        )

        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.deviceStateUpdate.category)
        )

        operationQueue.addOperation(operation)

        return operation
    }

    func setRelayConstraints(
        _ newConstraints: RelayConstraints,
        completionHandler: (() -> Void)? = nil
    ) {
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

    // MARK: - StorePaymentObserver

    func storePaymentManager(
        _ manager: StorePaymentManager,
        didReceiveEvent event: StorePaymentEvent
    ) {
        guard case let .finished(paymentCompletion) = event else {
            return
        }

        scheduleDeviceStateUpdate(
            taskName: "Update account expiry after in-app purchase",
            modificationBlock: { deviceState in
                switch deviceState {
                case .loggedIn(var accountData, let deviceData):
                    if accountData.number == paymentCompletion.accountNumber {
                        accountData.expiry = paymentCompletion.serverResponse.newExpiry
                        deviceState = .loggedIn(accountData, deviceData)
                    }

                case .loggedOut, .revoked:
                    break
                }
            },
            completionHandler: nil
        )
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
            refreshTunnelStatus()
        }
    }

    fileprivate func setTunnelStatus(_ block: (inout TunnelStatus) -> Void) -> TunnelStatus {
        nslock.lock()
        defer { nslock.unlock() }

        var newTunnelStatus = _tunnelStatus
        block(&newTunnelStatus)

        guard _tunnelStatus != newTunnelStatus else {
            return newTunnelStatus
        }

        logger.info("Status: \(newTunnelStatus).")

        _tunnelStatus = newTunnelStatus

        if let deviceCheck = newTunnelStatus.packetTunnelStatus.deviceCheck,
           deviceCheck.identifier != lastDeviceCheckIdentifier
        {
            if deviceCheck.isDeviceRevoked ?? false {
                didDetectDeviceRevoked()

            } else if let accountExpiry = deviceCheck.accountExpiry {
                scheduleDeviceStateUpdate(
                    taskName: "Update account expiry",
                    reconnectTunnel: false
                ) { deviceState in
                    if case .loggedIn(var accountData, let deviceData) = deviceState {
                        accountData.expiry = accountExpiry
                        deviceState = .loggedIn(accountData, deviceData)
                    }
                }
            }

            lastDeviceCheckIdentifier = deviceCheck.identifier
        }

        switch newTunnelStatus.state {
        case .connecting, .reconnecting:
            // Start polling tunnel status to keep the relay information up to date
            // while the tunnel process is trying to connect.
            startPollingTunnelStatus(interval: establishingTunnelStatusPollInterval)

        case .connected, .waitingForConnectivity:
            // Start polling tunnel status to keep connectivity status up to date.
            startPollingTunnelStatus(interval: establishedTunnelStatusPollInterval)

        case .pendingReconnect, .disconnecting, .disconnected:
            // Stop polling tunnel status once connection moved to final state.
            cancelPollingTunnelStatus()
        }

        DispatchQueue.main.async {
            self.observerList.forEach { observer in
                observer.tunnelManager(self, didUpdateTunnelStatus: newTunnelStatus)
            }
        }

        return newTunnelStatus
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
                    error: error,
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
                    error: error,
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

    @objc private func applicationDidBecomeActive(_ notification: Notification) {
        #if DEBUG
        logger.debug("Refresh tunnel status due to application becoming active.")
        #endif
        refreshTunnelStatus()
    }

    fileprivate func selectRelay() throws -> RelaySelectorResult {
        let cachedRelays = try relayCacheTracker.getCachedRelays()

        return try RelaySelector.evaluate(
            relays: cachedRelays.relays,
            constraints: settings.relayConstraints
        )
    }

    fileprivate func prepareForVPNConfigurationDeletion() {
        nslock.lock()
        defer { nslock.unlock() }

        // Unregister from receiving VPN connection status changes
        unsubscribeVPNStatusObserver()

        // Cancel last VPN status mapping operation
        lastMapConnectionStatusOperation?.cancel()
        lastMapConnectionStatusOperation = nil
    }

    private func checkIfDeviceRevoked(_ error: Error) {
        if let error = error as? REST.Error, error.compareErrorCode(.deviceNotFound) {
            didDetectDeviceRevoked()
        }
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

    private func didReconnectTunnel(completion: OperationCompletion<Void, Error>) {
        nslock.lock()
        defer { nslock.unlock() }

        if let error = completion.error {
            logger.error(
                error: error,
                message: "Failed to reconnect the tunnel."
            )
        }

        // Refresh tunnel status only when connecting or reasserting to pick up the next relay,
        // since both states may persist for a long period of time until the tunnel is fully
        // connected.
        switch tunnelStatus.state {
        case .connecting, .reconnecting:
            logger.debug("Refresh tunnel status due to reconnect.")
            refreshTunnelStatus()

        default:
            break
        }
    }

    private func subscribeVPNStatusObserver(tunnel: Tunnel) {
        nslock.lock()
        defer { nslock.unlock() }

        unsubscribeVPNStatusObserver()

        statusObserver = tunnel
            .addBlockObserver(queue: internalQueue) { [weak self] tunnel, status in
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

    private func refreshTunnelStatus() {
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
    ) {
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

        operation.addObserver(BackgroundObserver(
            application: application,
            name: taskName,
            cancelUponExpiration: false
        ))
        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.settingsUpdate.category)
        )

        operationQueue.addOperation(operation)
    }

    private func scheduleDeviceStateUpdate(
        taskName: String,
        reconnectTunnel: Bool = true,
        modificationBlock: @escaping (inout DeviceState) -> Void,
        completionHandler: (() -> Void)? = nil
    ) {
        let operation = AsyncBlockOperation(dispatchQueue: internalQueue) {
            var deviceState = self.deviceState

            modificationBlock(&deviceState)

            self.setDeviceState(deviceState, persist: true)

            if reconnectTunnel {
                self.reconnectTunnel(selectNewRelay: false, completionHandler: nil)
            }
        }

        operation.completionBlock = {
            DispatchQueue.main.async {
                completionHandler?()
            }
        }

        operation.addObserver(BackgroundObserver(
            application: application,
            name: taskName,
            cancelUponExpiration: false
        ))
        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.deviceStateUpdate.category)
        )

        operationQueue.addOperation(operation)
    }

    // MARK: - Tunnel status polling

    private func startPollingTunnelStatus(interval: TimeInterval) {
        guard !isPolling else { return }

        isPolling = true

        logger.debug(
            "Start polling tunnel status every \(interval) second(s)."
        )

        let timer = DispatchSource.makeTimerSource(queue: .main)
        timer.setEventHandler { [weak self] in
            self?.refreshTunnelStatus()
        }
        timer.schedule(wallDeadline: .now() + interval, repeating: interval)
        timer.activate()

        tunnelStatusPollTimer?.cancel()
        tunnelStatusPollTimer = timer
    }

    private func cancelPollingTunnelStatus() {
        guard isPolling else { return }

        logger.debug("Cancel tunnel status polling.")

        tunnelStatusPollTimer?.cancel()
        tunnelStatusPollTimer = nil
        isPolling = false
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

    func getPersistentTunnels() -> [Tunnel] {
        return tunnelManager.tunnelStore.getPersistentTunnels()
    }

    func createNewTunnel() -> Tunnel {
        return tunnelManager.tunnelStore.createNewTunnel()
    }

    func setTunnel(_ tunnel: Tunnel?, shouldRefreshTunnelState: Bool) {
        tunnelManager.setTunnel(tunnel, shouldRefreshTunnelState: shouldRefreshTunnelState)
    }

    var tunnelStatus: TunnelStatus {
        return tunnelManager.tunnelStatus
    }

    func updateTunnelStatus(_ block: (inout TunnelStatus) -> Void) -> TunnelStatus {
        return tunnelManager.setTunnelStatus(block)
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

    func selectRelay() throws -> RelaySelectorResult {
        return try tunnelManager.selectRelay()
    }
}
