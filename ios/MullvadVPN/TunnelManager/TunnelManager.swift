// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import NetworkExtension
import Operations
import PacketTunnelCore
import UIKit

/// Interval used for periodic polling of tunnel relay status when the packet tunnel is running
private let tunnelStatusPollInterval: Duration = .milliseconds(500)

/// An actor that provides a convenient interface for VPN tunnels configuration, manipulation and
/// monitoring.
/// Assumes safe for isolation in various places (mainly related to operations) as long work is
/// done on `internalQueue`.
actor TunnelManager {
    private enum OperationCategory: String, Sendable {
        case manageTunnel
        case deviceStateUpdate
        case settingsUpdate

        var category: String {
            "TunnelManager.\(rawValue)"
        }
    }

    // MARK: - Internal variables

    let backgroundTaskProvider: BackgroundTaskProviding
    let relaySelector: RelaySelectorProtocol
    // `let` is implicitly nonisolated on actors, allowing use as a custom executor in
    // nonisolated contexts.
    let internalQueue = DispatchSerialQueue(label: "TunnelManager.internalQueue")

    fileprivate let tunnelStore: any TunnelStoreProtocol
    private let relayCacheTracker: RelayCacheTrackerProtocol
    private let accountsProxy: RESTAccountHandling
    private let devicesProxy: DeviceHandling
    private let apiProxy: APIQuerying
    private let logger = Logger(label: "TunnelManager")
    private let operationQueue = AsyncOperationQueue()

    private var statusObserver: TunnelStatusBlockObserver?
    nonisolated let observerList = ObserverList<TunnelObserver>()
    private var networkMonitor: NWPathMonitor?
    private var pendingNetworkPathUpdate: DispatchWorkItem?
    private static let networkPathUpdateDelay: DispatchTimeInterval = .seconds(5)

    private var privateKeyRotationTimer: DispatchSourceTimer?
    public private(set) var isRunningPeriodicPrivateKeyRotation = false
    public private(set) var nextKeyRotationDate: Date?

    private var tunnelStatusPollTimer: DispatchSourceTimer?
    private var isPolling = false
    /// Last processed device check.
    private var lastPacketTunnelKeyRotation: Date?
    private var observer: TunnelObserver?
    private let settingsManager: SettingsManager

    fileprivate var _isConfigurationLoaded = false
    fileprivate var _deviceState: DeviceState = .loggedOut
    fileprivate var _tunnelSettings = LatestTunnelSettings()
    fileprivate var _tunnel: (any TunnelProtocol)?
    fileprivate var _tunnelStatus = TunnelStatus()
    fileprivate var _lastNEVPNStatus: NEVPNStatus = .invalid

    nonisolated(unsafe) private var accountManager: AccountManager!

    // MARK: - Custom executor

    nonisolated var unownedExecutor: UnownedSerialExecutor {
        internalQueue.asUnownedSerialExecutor()
    }

    // MARK: - Initialization

    init(
        backgroundTaskProvider: BackgroundTaskProviding,
        tunnelStore: any TunnelStoreProtocol,
        relayCacheTracker: RelayCacheTrackerProtocol,
        accountsProxy: RESTAccountHandling,
        devicesProxy: DeviceHandling,
        apiProxy: APIQuerying,
        relaySelector: RelaySelectorProtocol,
        settingsManager: SettingsManager
    ) {
        self.backgroundTaskProvider = backgroundTaskProvider
        self.tunnelStore = tunnelStore
        self.relayCacheTracker = relayCacheTracker
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
        self.apiProxy = apiProxy
        self.operationQueue.name = "TunnelManager.operationQueue"
        self.operationQueue.underlyingQueue = internalQueue
        self.relaySelector = relaySelector
        self.settingsManager = settingsManager

        self.accountManager = AccountManager(
            operationQueue: operationQueue,
            internalQueue: internalQueue,
            interactor: TunnelInteractorProxy(self),
            backgroundTaskProvider: backgroundTaskProvider,
            accountsProxy: accountsProxy,
            devicesProxy: devicesProxy,
            category: OperationCategory.deviceStateUpdate.category
        )

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(applicationDidBecomeActive),
            name: UIApplication.didBecomeActiveNotification,
            object: nil
        )

        Task { [weak self] in
            await self?.startNetworkMonitor()
        }

        #if NOT_PRODUCTION_PERMANENT
            relayCacheTracker.addObserver(self)
        #endif
    }

    // MARK: - Account

    func setNewAccount() async throws -> StoredAccountData {
        try await setAccount(action: .new)!
    }

    func setExistingAccount(accountNumber: String) async throws -> StoredAccountData {
        try await setAccount(action: .existing(accountNumber))!
    }

    func unsetAccount(isRemovingProfile: Bool = true) async {
        do {
            _ = try await setAccount(action: .unset)
            await unsetTunnelConfiguration(isRemovingProfile: isRemovingProfile)
        } catch {
            logger.debug("Failed to unset account: \(error.description)")
        }
    }

    func updateAccountData() async throws {
        try await accountManager.updateAccountData()
    }

    func deleteAccount(accountNumber: String) async throws {
        _ = try await setAccount(action: .delete(accountNumber))
        removeLastUsedAccount()
        await unsetTunnelConfiguration()
    }

    func updateDeviceData() async throws {
        try await accountManager.updateDeviceData()
    }

    private func setAccount(action: SetAccountAction) async throws -> StoredAccountData? {
        try await withCheckedThrowingContinuation { continuation in
            accountManager.setAccount(action: action) { [weak self] result in
                Task {
                    await self?.startOrStopPeriodicPrivateKeyRotation()
                    continuation.resume(with: result)
                }
            }
        }
    }

    func rotatePrivateKey(
        completionHandler: @MainActor @escaping @Sendable (Error?) async -> Void
    ) async -> Cancellable {
        await accountManager.rotatePrivateKey { [weak self] result in
            Task {
                guard let self else { return }

                await self.updatePrivateKeyRotationTimer()

                switch result {
                case .success:
                    await completionHandler(await self.notifyTunnelKeyRotated())
                case .failure(let error):
                    await self.handleRestError(error)
                    await completionHandler(error)
                }
            }
        }
    }

    private func notifyTunnelKeyRotated() async -> Error? {
        await withCheckedContinuation { continuation in
            _ = _tunnel?.notifyKeyRotation { result in
                continuation.resume(returning: result.error)
            }
        }
    }

    // MARK: - Periodic private key rotation

    func startPeriodicPrivateKeyRotation() {
        guard !isRunningPeriodicPrivateKeyRotation, _deviceState.isLoggedIn else { return }

        logger.debug("Start periodic private key rotation.")

        isRunningPeriodicPrivateKeyRotation = true
        updatePrivateKeyRotationTimer()
    }

    func stopPeriodicPrivateKeyRotation() {
        guard isRunningPeriodicPrivateKeyRotation else { return }

        logger.debug("Stop periodic private key rotation.")

        isRunningPeriodicPrivateKeyRotation = false
        updatePrivateKeyRotationTimer()
    }

    func startOrStopPeriodicPrivateKeyRotation() {
        if _deviceState.isLoggedIn {
            startPeriodicPrivateKeyRotation()
        } else {
            stopPeriodicPrivateKeyRotation()
        }
    }

    func getNextKeyRotationDate() -> Date? {
        _deviceState.deviceData.flatMap { WgKeyRotation(data: $0).nextRotationDate }
    }

    private func updatePrivateKeyRotationTimer() {
        privateKeyRotationTimer?.cancel()
        privateKeyRotationTimer = nil
        nextKeyRotationDate = nil

        guard isRunningPeriodicPrivateKeyRotation,
            let scheduleDate = getNextKeyRotationDate()
        else { return }
        nextKeyRotationDate = scheduleDate

        let timer = DispatchSource.makeTimerSource(queue: .main)

        timer.setEventHandler { [weak self] in
            Task {
                _ = await self?.accountManager.rotatePrivateKey { _ in }
            }
        }

        timer.schedule(wallDeadline: .now() + scheduleDate.timeIntervalSinceNow)
        timer.activate()

        privateKeyRotationTimer = timer

        logger.debug("Schedule next private key rotation at \(scheduleDate.logFormatted).")
    }

    // MARK: - Public methods

    func loadConfiguration() async {
        let task = LoadTunnelConfigurationTask(
            interactor: TunnelInteractorProxy(self),
            settingsManager: settingsManager)

        /// Keep an `AsyncOperation` around to keep the same exclusivity behaviour until
        /// `TunnelManager` is migrated away from `AsyncOperation` code
        let loadTunnelOperation = AsyncBlockOperation(dispatchQueue: internalQueue) { [weak self] in
            Task {
                await task.start()
                await self?.updatePrivateKeyRotationTimer()
            }
        }
        loadTunnelOperation.addObserver(
            BackgroundObserver(
                backgroundTaskProvider: backgroundTaskProvider,
                name: "Load tunnel configuration",
                cancelUponExpiration: false
            )
        )
        loadTunnelOperation.addCondition(MutuallyExclusive(category: OperationCategory.manageTunnel.category))
        operationQueue.addOperation(loadTunnelOperation)
    }

    func startTunnel(completionHandler: ((Error?) -> Void)? = nil) {
        let operation = StartTunnelOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self),
            completionHandler: { [weak self] result in
                guard let self else { return }
                if let error = result.error {
                    self.logger.error(
                        error: error,
                        message: "Failed to start the tunnel."
                    )

                    let tunnelError = StartTunnelError(underlyingError: error)

                    self.observerList.notify { observer in
                        observer.tunnelManager(self, didFailWithError: tunnelError)
                    }
                }

                completionHandler?(result.error)
            }
        )

        operation.addObserver(
            BackgroundObserver(
                backgroundTaskProvider: backgroundTaskProvider,
                name: "Start tunnel",
                cancelUponExpiration: true
            ))
        operation.addCondition(MutuallyExclusive(category: OperationCategory.manageTunnel.category))

        operationQueue.addOperation(operation)
    }

    func stopTunnel(isOnDemandEnabled: Bool = false, completionHandler: ((Error?) -> Void)? = nil) {
        let operation = StopTunnelOperation(
            dispatchQueue: internalQueue,
            interactor: TunnelInteractorProxy(self)
        ) { [weak self] result in
            guard let self else { return }

            if let error = result.error {
                self.logger.error(
                    error: error,
                    message: "Failed to stop the tunnel."
                )

                let tunnelError = StopTunnelError(underlyingError: error)

                self.observerList.notify { observer in
                    observer.tunnelManager(self, didFailWithError: tunnelError)
                }
            }

            completionHandler?(result.error)
        }
        operation.isOnDemandEnabled = isOnDemandEnabled
        operation.addObserver(
            BackgroundObserver(
                backgroundTaskProvider: backgroundTaskProvider,
                name: "Stop tunnel",
                cancelUponExpiration: true
            ))
        operation.addCondition(MutuallyExclusive(category: OperationCategory.manageTunnel.category))

        operationQueue.addOperation(operation)
    }

    func reconnectTunnel(selectNewRelay: Bool, completionHandler: (@Sendable (Error?) -> Void)? = nil) {
        // Start polling the tunnel immediately when the user reconnects
        startPollingTunnelStatus(interval: tunnelStatusPollInterval)

        let operation = AsyncBlockOperation(dispatchQueue: internalQueue) { [self] finish in
            self.assumeIsolated { actor in
                guard let tunnel = actor._tunnel else {
                    finish(UnsetTunnelError())
                    return
                }

                _ = tunnel.reconnectTunnel(to: selectNewRelay ? .random : .current) { result in
                    Task { [weak self] in
                        if case let .success(observedState) = result,
                            let connectionState = observedState.connectionState,
                            let self
                        {
                            _ = await setTunnelStatus { tunnelStatus in
                                tunnelStatus.state = .reconnecting(
                                    connectionState.selectedRelays,
                                    isPostQuantum: connectionState.isPostQuantum,
                                    isDaita: connectionState.isDaitaEnabled
                                )
                                tunnelStatus.observedState = observedState
                            }
                        }

                        finish(result.error)
                    }
                }
            }
        }

        operation.completionBlock = { [weak self] in
            Task { [weak self] in
                await self?.didReconnectTunnel(error: operation.error)
                completionHandler?(operation.error)
            }
        }

        operation.addObserver(
            BackgroundObserver(
                backgroundTaskProvider: backgroundTaskProvider,
                name: "Reconnect tunnel",
                cancelUponExpiration: true
            )
        )
        operation.addCondition(MutuallyExclusive(category: OperationCategory.manageTunnel.category))

        operationQueue.addOperation(operation)
    }

    func reapplyTunnelConfiguration() async {
        guard let tunnel = _tunnel else { return }

        if _tunnelStatus.state.isSecured {
            let observer = TunnelBlockObserver(
                didUpdateTunnelStatus: { [weak self] _, status in
                    Task { [weak self] in
                        if case .disconnected = status.state {
                            await self?.clearObserver()
                            await self?.startTunnel()
                        }
                    }
                }
            )

            addObserver(observer)
            self.observer = observer

            let configuration = TunnelConfiguration(
                includeAllNetworks: _tunnelSettings.includeAllNetworks.includeAllNetworksIsEnabled,
                excludeLocalNetworks: _tunnelSettings.includeAllNetworks.localNetworkSharingIsEnabled
            )

            await tunnel.setConfiguration(configuration)
            _ = await tunnel.saveToPreferences()
            stopTunnel(isOnDemandEnabled: true)
        }
    }

    func updateSettings(_ updates: [TunnelSettingsUpdate]) async {
        let taskName = "Set " + updates.map(\.subjectName).joined(separator: ", ")

        await withCheckedContinuation { continuation in
            scheduleSettingsUpdate(
                taskName: taskName,
                modificationBlock: { settings in
                    for update in updates {
                        update.apply(to: &settings)
                    }
                },
                completionHandler: {
                    continuation.resume()
                }
            )
        }
    }

    func refreshRelayCacheTracker() throws {
        try relayCacheTracker.refreshCachedRelays()
    }

    func selectRelays(tunnelSettings: LatestTunnelSettings) throws -> SelectedRelays {
        let retryAttempts = _tunnelStatus.observedState.connectionState?.connectionAttemptCount ?? 0

        return try relaySelector.selectRelays(
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: retryAttempts
        )
    }

    fileprivate func selectRelays() throws -> SelectedRelays {
        try selectRelays(tunnelSettings: _tunnelSettings)
    }

    // MARK: - Tunnel observation

    /// Add tunnel observer.
    /// In order to cancel the observation, either call `removeObserver(_:)` or simply release
    /// the observer.
    /// Nonisolation is safe long as `ObserverList.addend` keeps its locks.
    nonisolated func addObserver(_ observer: TunnelObserver) {
        observerList.append(observer)
    }

    /// Remove tunnel observer.
    /// Nonisolation is safe long as `ObserverList.remove` keeps its locks.
    nonisolated func removeObserver(_ observer: TunnelObserver) {
        observerList.remove(observer)
    }

    private func clearObserver() {
        if let observer = observer {
            removeObserver(observer)
        }

        observer = nil
    }

    // MARK: - TunnelInteractor

    /// Nonisolation is safe due to execution on internal queue.
    nonisolated var isConfigurationLoaded: Bool {
        internalQueue.sync { assumeIsolated { actor in actor._isConfigurationLoaded } }
    }

    /// Nonisolation is safe due to execution on internal queue.
    nonisolated var tunnelStatus: TunnelStatus {
        internalQueue.sync { assumeIsolated { $0._tunnelStatus } }
    }

    /// Nonisolation is safe due to execution on internal queue.
    nonisolated var settings: LatestTunnelSettings {
        internalQueue.sync { assumeIsolated { $0._tunnelSettings } }
    }

    /// Nonisolation is safe due to execution on internal queue.
    nonisolated var deviceState: DeviceState {
        internalQueue.sync { assumeIsolated { $0._deviceState } }
    }

    fileprivate var tunnel: (any TunnelProtocol)? {
        _tunnel
    }

    fileprivate func setConfigurationLoaded() {
        guard !_isConfigurationLoaded else { return }

        _isConfigurationLoaded = true

        observerList.notify { observer in
            Task { @MainActor in
                observer.tunnelManagerDidLoadConfiguration(self)
            }
        }
    }

    fileprivate func setTunnel(_ tunnel: (any TunnelProtocol)?, shouldRefreshTunnelState: Bool) async {
        if let tunnel {
            await subscribeVPNStatusObserver(tunnel: tunnel)
        } else {
            unsubscribeVPNStatusObserver()
        }

        _tunnel = tunnel

        // Update the existing state
        if shouldRefreshTunnelState {
            logger.debug("Refresh tunnel status for new tunnel.")
            await refreshTunnelStatus()
        }
    }

    @discardableResult
    fileprivate func setTunnelStatus(_ block: @Sendable (inout TunnelStatus) -> Void) async -> TunnelStatus {
        var newTunnelStatus = _tunnelStatus
        block(&newTunnelStatus)

        guard _tunnelStatus != newTunnelStatus else {
            return newTunnelStatus
        }

        logger.info("Status: \(newTunnelStatus).")

        _tunnelStatus = newTunnelStatus

        // Packet tunnel may have attempted or rotated the key.
        // In that case we have to reload device state from Keychain as it's likely was modified by packet tunnel.
        let newPacketTunnelKeyRotation = _tunnelStatus.observedState.connectionState?.lastKeyRotation
        if lastPacketTunnelKeyRotation != newPacketTunnelKeyRotation {
            lastPacketTunnelKeyRotation = newPacketTunnelKeyRotation
            refreshDeviceState()
        }
        // Handle unrecoverable blocked states
        if case let .error(blockedStateReason) = _tunnelStatus.state,
            !blockedStateReason.recoverableError()
        {
            await handleBlockedState(reason: blockedStateReason)
        }

        let snapshot = newTunnelStatus
        observerList.notify { observer in
            Task { @MainActor in
                observer.tunnelManager(self, didUpdateTunnelStatus: snapshot)
            }
        }

        return newTunnelStatus
    }

    fileprivate func setSettings(_ settings: LatestTunnelSettings, persist: Bool) {
        let shouldCallDelegate = _tunnelSettings != settings && _isConfigurationLoaded

        _tunnelSettings = settings

        if persist {
            do {
                try settingsManager.writeSettings(settings)
            } catch {
                logger.error(
                    error: error,
                    message: "Failed to write settings."
                )
            }
        }

        if shouldCallDelegate {
            observerList.notify { observer in
                Task { @MainActor in
                    observer.tunnelManager(self, didUpdateTunnelSettings: settings)
                }
            }
        }
    }

    func setDeviceState(_ deviceState: DeviceState, persist: Bool) {
        let shouldCallDelegate = _deviceState != deviceState && _isConfigurationLoaded
        let previousDeviceState = _deviceState

        _deviceState = deviceState

        if persist {
            do {
                try settingsManager.writeDeviceState(deviceState)
            } catch {
                logger.error(
                    error: error,
                    message: "Failed to write device state."
                )
            }
        }

        if shouldCallDelegate {
            observerList.notify { observer in
                Task { @MainActor [weak self] in
                    guard let self else { return }

                    observer.tunnelManager(
                        self,
                        didUpdateDeviceState: deviceState,
                        previousDeviceState: previousDeviceState
                    )
                }
            }
        }
    }

    // MARK: - Private methods

    /// Nonisolation is safe as long as no actual actor-isolated work is performed outside
    /// the actor's executor.
    @objc nonisolated private func applicationDidBecomeActive() {
        #if DEBUG
            logger.debug(
                "Refresh device state and tunnel status due to application becoming active."
            )
        #endif

        Task { [weak self] in
            await self?.refreshTunnelStatus()
            await self?.refreshDeviceState()
        }
    }

    private func didUpdateNetworkPath(_ path: Network.NWPath) async {
        // Only act on network path updates when VPN is disconnected.
        // When VPN is up, the packet tunnel handles network changes internally.
        let status = await _tunnel?.status ?? .disconnected
        guard [.disconnected, .invalid].contains(status) else { return }

        await setDisconnectedState(networkPathStatus: path.status)
    }

    fileprivate func prepareForVPNConfigurationDeletion() {
        unsubscribeVPNStatusObserver()
    }

    private func didReconnectTunnel(error: Error?) async {
        if let error, !error.isOperationCancellationError {
            logger.error(error: error, message: "Failed to reconnect the tunnel.")
        }

        // Refresh tunnel status only when connecting, reasserting or error to pick up the next relay,
        // since both states may persist for a long period of time until the tunnel is fully connected.
        switch _tunnelStatus.state {
        case .connecting, .reconnecting, .error:
            logger.debug("Refresh tunnel status due to reconnect.")
            await refreshTunnelStatus()

        default:
            break
        }
    }

    private func subscribeVPNStatusObserver(tunnel: any TunnelProtocol) async {
        unsubscribeVPNStatusObserver()

        // Refresh tunnel status only when connecting, reasserting or error to pick up the next relay,
        // since both states may persist for a long period of time until the tunnel is fully connected.
        switch _tunnelStatus.state {
        case .connecting, .reconnecting, .error:
            logger.debug("Refresh tunnel status due to reconnect.")
            refreshTunnelStatus()

        default:
            break
        }
    }

    private func subscribeVPNStatusObserver(tunnel: any TunnelProtocol) async {
        unsubscribeVPNStatusObserver()

        statusObserver = tunnel.addBlockObserver(queue: internalQueue) { [weak self] _, status in
            self?.assumeIsolated { actor in
                // Save the NEVPNStatus so we can reject stale IPC updates
                actor.setNEVPNStatus(status)

                // Control polling based on NEVPNStatus directly (the source of truth),
                // not the derived tunnelStatus.state which can be stale.
                actor.updatePollingFromVPNStatus(status)

                // Update tunnel status for all state changes to ensure UI reflects
                // disconnecting and disconnected states immediately.
                actor.updateTunnelStatus(status)
            }
        }

        // Save and start polling for the current status since the observer
        // only fires on status changes, not for the initial state.
        setNEVPNStatus(await tunnel.status)
        updatePollingFromVPNStatus(await tunnel.status)
    }

    private func setNEVPNStatus(_ status: NEVPNStatus) {
        _lastNEVPNStatus = status
        logger.debug("VPN connection status changed to \(status).")
    }

    private func startNetworkMonitor() {
        networkMonitor = NWPathMonitor()
        networkMonitor?.pathUpdateHandler = { [weak self] path in
            self?.assumeIsolated { actor in
                actor.scheduleNetworkPathUpdate(path)
            }
        }

        networkMonitor?.start(queue: internalQueue)
    }

    /// Schedule a network path update with a delay to debounce rapid changes.
    private func scheduleNetworkPathUpdate(_ path: Network.NWPath) {
        pendingNetworkPathUpdate?.cancel()

        let workItem = DispatchWorkItem { [weak self] in
            Task {
                await self?.didUpdateNetworkPath(path)
            }
        }
        pendingNetworkPathUpdate = workItem

        internalQueue.asyncAfter(
            deadline: .now().advanced(by: Self.networkPathUpdateDelay),
            execute: workItem
        )
    }

    private func unsubscribeVPNStatusObserver() {
        statusObserver?.invalidate()
        statusObserver = nil
    }

    private func refreshTunnelStatus() async {
        guard let connectionStatus = await _tunnel?.status else { return }

        switch connectionStatus {
        case .connecting, .reasserting, .connected:
            // Active states: fetch via IPC
            fetchAndUpdateTunnelStatus()
        case .disconnected, .disconnecting, .invalid:
            // Down states: update directly
            await updateTunnelStatus(connectionStatus)
        @unknown default:
            break
        }
    }

    /// Refresh device state from settings and update the in-memory value.
    /// Used to refresh device state when it's modified by packet tunnel during key rotation.
    private func refreshDeviceState() {
        let operation = AsyncBlockOperation(dispatchQueue: internalQueue) { [self, settingsManager] in
            do {
                let newDeviceState = try settingsManager.readDeviceState()

                self.assumeIsolated { actor in
                    actor.setDeviceState(newDeviceState, persist: false)
                }
            } catch {
                if let error = error as? KeychainError, error == .itemNotFound {
                    return
                }
                logger.error(error: error, message: "Failed to refresh device state")
            }
        }

        operation.addCondition(MutuallyExclusive(category: OperationCategory.deviceStateUpdate.category))
        operation.addObserver(
            BackgroundObserver(
                backgroundTaskProvider: backgroundTaskProvider,
                name: "Refresh device state",
                cancelUponExpiration: true
            ))

        operationQueue.addOperation(operation)
    }

    /// Update `TunnelStatus` from `NEVPNStatus`.
    /// For active states, fetches detailed status via IPC.
    /// For down states, updates state directly without IPC.
    private func updateTunnelStatus(_ connectionStatus: NEVPNStatus) async {
        switch connectionStatus {
        case .connecting, .reasserting, .connected:
            // Active states: fetch details via IPC
            fetchAndUpdateTunnelStatus()

        case .disconnecting:
            await handleDisconnectingStateDirectly()

        case .disconnected:
            await handleDisconnectedStateDirectly()

        case .invalid:
            await setDisconnectedState(networkPathStatus: networkMonitor?.currentPath.status)

        @unknown default:
            logger.debug("Unknown NEVPNStatus: \(connectionStatus.rawValue)")
        }
    }

    // MARK: - Direct state updates (no IPC needed)

    /// Directly set disconnected state without going through operation queue.
    /// Safe because disconnected is an idempotent final state.
    private func setDisconnectedState(networkPathStatus: Network.NWPath.Status?) async {
        _ = await setTunnelStatus { tunnelStatus in
            tunnelStatus = TunnelStatus()
            tunnelStatus.state =
                networkPathStatus == .unsatisfied
                ? .waitingForConnectivity(.noNetwork)
                : .disconnected
        }
    }

    /// Handle disconnecting state directly without IPC.
    private func handleDisconnectingStateDirectly() async {
        switch _tunnelStatus.state {
        case .disconnecting:
            // Already disconnecting, no change needed
            break
        default:
            _ = await setTunnelStatus { tunnelStatus in
                if tunnelStatus.observedState.blockedState != nil {
                    tunnelStatus.state = .disconnecting(.nothing)
                } else {
                    let isNetworkReachable = tunnelStatus.observedState.connectionState?.isNetworkReachable ?? false
                    tunnelStatus.state =
                        isNetworkReachable
                        ? .disconnecting(.nothing)
                        : .waitingForConnectivity(.noNetwork)
                }
            }
        }
    }

    /// Handle disconnected state directly without IPC.
    private func handleDisconnectedStateDirectly() async {
        switch _tunnelStatus.state {
        case .pendingReconnect:
            logger.debug("Ignore disconnected state when pending reconnect.")

        case .disconnecting(.reconnect):
            logger.debug("Restart the tunnel on disconnect.")
            _ = await setTunnelStatus { tunnelStatus in
                tunnelStatus = TunnelStatus()
                tunnelStatus.state = .pendingReconnect
            }
            startTunnel()

        default:
            await setDisconnectedState(networkPathStatus: networkMonitor?.currentPath.status)
        }
    }

    // MARK: - IPC-based status fetch (for active states)

    /// Fetch tunnel status via IPC and update state.
    /// Only called for active states (.connecting, .reasserting, .connected).
    private func fetchAndUpdateTunnelStatus() {
        guard let tunnel = _tunnel else { return }

        _ = tunnel.getTunnelStatus { [weak self] result in
            guard let self else { return }

            Task {
                // Reject stale IPC updates if the tunnel is now dead
                guard await isTunnelAlive else {
                    logger.debug("Ignoring stale IPC response, tunnel is dead.")
                    return
                }

                if case let .success(observedState) = result {
                    let newState = await mapObservedStateToTunnelState(observedState)
                    _ = await setTunnelStatus { tunnelStatus in
                        tunnelStatus.observedState = observedState
                        if let state = newState {
                            tunnelStatus.state = state
                        }
                    }
                }
            }
        }
    }

    /// Returns true if the tunnel is in an active state based on NEVPNStatus.
    private var isTunnelAlive: Bool {
        switch _lastNEVPNStatus {
        case .connecting, .reasserting, .connected:
            return true
        case .disconnecting, .disconnected, .invalid:
            return false
        @unknown default:
            return false
        }
    }

    /// Map ObservedState from packet tunnel to TunnelState for UI.
    private func mapObservedStateToTunnelState(_ observedState: ObservedState) -> TunnelState? {
        switch observedState {
        case let .connected(connectionState):
            return .connected(
                connectionState.selectedRelays,
                isPostQuantum: connectionState.isPostQuantum,
                isDaita: connectionState.isDaitaEnabled
            )
        case let .connecting(connectionState):
            return .connecting(
                connectionState.selectedRelays,
                isPostQuantum: connectionState.isPostQuantum,
                isDaita: connectionState.isDaitaEnabled
            )
        case let .negotiatingEphemeralPeer(connectionState, privateKey):
            return .negotiatingEphemeralPeer(
                connectionState.selectedRelays,
                privateKey,
                isPostQuantum: connectionState.isPostQuantum,
                isDaita: connectionState.isDaitaEnabled
            )
        case let .reconnecting(connectionState):
            return .reconnecting(
                connectionState.selectedRelays,
                isPostQuantum: connectionState.isPostQuantum,
                isDaita: connectionState.isDaitaEnabled
            )
        case let .error(blockedState):
            return .error(blockedState.reason)
        case .initial, .disconnecting, .disconnected:
            return nil
        }
    }

    private func scheduleSettingsUpdate(
        taskName: String,
        modificationBlock: @escaping @Sendable (inout LatestTunnelSettings) -> Void,
        completionHandler: (@Sendable () -> Void)?
    ) {
        let operation = AsyncBlockOperation(dispatchQueue: internalQueue) { [weak self] in
            self?.assumeIsolated { actor in
                let currentSettings = actor._tunnelSettings
                var updatedSettings = actor._tunnelSettings
                let settingsStrategy = TunnelSettingsStrategy()

                modificationBlock(&updatedSettings)
                actor.setSettings(updatedSettings, persist: true)

                let reconnectionStrategy = settingsStrategy.getReconnectionStrategy(
                    oldSettings: currentSettings,
                    newSettings: updatedSettings
                )

                switch reconnectionStrategy {
                case .currentRelayReconnect:
                    actor.reconnectTunnel(selectNewRelay: false)
                case .newRelayReconnect:
                    actor.reconnectTunnel(selectNewRelay: true)
                case .hardReconnect:
                    Task {
                        await actor.reapplyTunnelConfiguration()
                    }
                }
            }
        }

        operation.completionBlock = {
            Task { @MainActor in
                completionHandler?()
            }
        }

        operation.addObserver(
            BackgroundObserver(
                backgroundTaskProvider: backgroundTaskProvider,
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
        modificationBlock: @escaping @Sendable (inout DeviceState) -> Void,
        completionHandler: (@Sendable () -> Void)? = nil
    ) {
        let operation = AsyncBlockOperation(dispatchQueue: internalQueue) { [weak self] in
            self?.assumeIsolated { actor in
                var deviceState = actor._deviceState

                modificationBlock(&deviceState)

                actor.setDeviceState(deviceState, persist: true)

                if reconnectTunnel {
                    actor.reconnectTunnel(selectNewRelay: false, completionHandler: nil)
                }
            }
        }

        operation.completionBlock = {
            Task { @MainActor in
                completionHandler?()
            }
        }

        operation.addObserver(
            BackgroundObserver(
                backgroundTaskProvider: backgroundTaskProvider,
                name: taskName,
                cancelUponExpiration: false
            ))
        operation.addCondition(
            MutuallyExclusive(category: OperationCategory.deviceStateUpdate.category)
        )

        operationQueue.addOperation(operation)
    }

    // MARK: - Tunnel status polling

    private func startPollingTunnelStatus(interval: Duration) {
        guard !isPolling else {
            return
        }

        isPolling = true
        tunnelStatusPollTimer?.cancel()

        logger.debug("Start polling tunnel status every \(interval.logFormat()).")

        let timer = DispatchSource.makeTimerSource(queue: .main)
        timer.setEventHandler { [weak self] in
            Task { [weak self] in
                await self?.refreshTunnelStatus()
            }
        }
        timer.schedule(wallDeadline: .now() + interval, repeating: interval.timeInterval)
        timer.activate()

        tunnelStatusPollTimer = timer
    }

    private func cancelPollingTunnelStatus() {
        guard isPolling else { return }

        logger.debug("Cancel tunnel status polling.")

        tunnelStatusPollTimer?.cancel()
        tunnelStatusPollTimer = nil
        isPolling = false
    }

    /// Update polling state based on NEVPNStatus (the source of truth).
    /// Poll continuously while tunnel is not disconnected, disconnecting, or invalid.
    private func updatePollingFromVPNStatus(_ status: NEVPNStatus) {
        switch status {
        case .disconnecting, .disconnected, .invalid:
            cancelPollingTunnelStatus()
        case .connecting, .reasserting, .connected:
            startPollingTunnelStatus(interval: tunnelStatusPollInterval)
        @unknown default:
            // For any unknown future states, assume it's an active state and poll
            startPollingTunnelStatus(interval: tunnelStatusPollInterval)
        }
    }

    fileprivate func setLastUsedAccount(_ accountNumber: String) {
        logger.debug("Store last used account.")
        do {
            try settingsManager.setLastUsedAccount(accountNumber)
        } catch {
            logger.error(
                error: error,
                message: "Failed to store last used account number."
            )
        }
    }

    fileprivate func removeLastUsedAccount() {
        do {
            try settingsManager.setLastUsedAccount(nil)
        } catch {
            logger.error(
                error: error,
                message: "Failed to delete account data."
            )
        }
    }

    func handleRestError(_ error: Error) async {
        guard let restError = error as? REST.Error else { return }

        if restError.compareErrorCode(.deviceNotFound) {
            await handleBlockedState(reason: .deviceRevoked)
        } else if restError.compareErrorCode(.invalidAccount) {
            await handleBlockedState(reason: .invalidAccount)
        }
    }

    private func handleBlockedState(reason: BlockedStateReason) async {
        switch reason {
        case .deviceRevoked:
            setDeviceState(.revoked, persist: true)
        case .invalidAccount:
            setDeviceState(.revoked, persist: true)
            operationQueue.cancelAllOperations()
            removeLastUsedAccount()
            await unsetTunnelConfiguration()
        default:
            break
        }
    }

    fileprivate func unsetTunnelConfiguration(isRemovingProfile: Bool = true) async {
        prepareForVPNConfigurationDeletion()

        _ = await setTunnelStatus { tunnelStatus in
            tunnelStatus = TunnelStatus()
            tunnelStatus.state = .disconnected
        }

        guard let tunnel = _tunnel, isRemovingProfile else { return }

        if let error = await tunnel.removeFromPreferences() {
            logger.error(
                error: error,
                message: "Failed to remove VPN configuration."
            )
        }
        await setTunnel(nil, shouldRefreshTunnelState: false)
    }
}

#if DEBUG

    // MARK: - Simulations

    extension TunnelManager {
        enum AccountExpirySimulationOption {
            case closeToExpiry(days: Int)
            case expired
            case active

            fileprivate var date: Date? {
                let calendar = Calendar.current
                let now = Date()

                switch self {
                case .active:
                    return calendar.date(byAdding: .year, value: 1, to: now)

                case let .closeToExpiry(days):
                    return calendar.date(
                        byAdding: DateComponents(day: days, second: 5),
                        to: now
                    )

                case .expired:
                    return calendar.date(byAdding: .minute, value: -1, to: now)
                }
            }
        }

        /**

         This function simulates account state transitions. The change is not permanent and any call to
         `updateAccountData()` will overwrite it, but it's usually enough for quick testing.

         It can be invoked somewhere in `initTunnelManagerOperation` (`AppDelegate`) after tunnel manager is fully
         initialized. The following code snippet can be used to cycle through various states:

         ```
         func delay(seconds: UInt) async throws {
         try await Task.sleep(nanoseconds: UInt64(seconds) * 1_000_000_000)
         }

         Task {
         print("Wait 5 seconds")
         try await delay(seconds: 5)

         print("Simulate active account")
         self.tunnelManager.simulateAccountExpiration(option: .active)
         try await delay(seconds: 5)

         print("Simulate close to expiry")
         self.tunnelManager.simulateAccountExpiration(option: .closeToExpiry)
         try await delay(seconds: 10)

         print("Simulate expired account")
         self.tunnelManager.simulateAccountExpiration(option: .expired)
         try await delay(seconds: 5)

         print("Simulate active account")
         self.tunnelManager.simulateAccountExpiration(option: .active)
         }
         ```

         Another way to invoke this code is to pause debugger and run it directly:

         ```
         command alias swift expression -l Swift -O --

         swift import MullvadVPN
         swift (UIApplication.shared.delegate as? AppDelegate)?.tunnelManager.simulateAccountExpiration(option: .closeToExpiry)
         ```

         */
        func simulateAccountExpiration(option: AccountExpirySimulationOption) {
            scheduleDeviceStateUpdate(taskName: "Simulating account expiry", reconnectTunnel: false) { deviceState in
                guard case .loggedIn(var accountData, let deviceData) = deviceState, let date = option.date else {
                    return
                }

                accountData.expiry = date

                deviceState = .loggedIn(accountData, deviceData)
            }
        }
    }

#endif

private struct TunnelInteractorProxy: TunnelInteractor {
    private let tunnelManager: TunnelManager

    init(_ tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
    }

    func getTunnel() async -> (any TunnelProtocol)? {
        await tunnelManager._tunnel
    }

    var backgroundTaskProvider: BackgroundTaskProviding {
        tunnelManager.backgroundTaskProvider
    }

    func getPersistentTunnel() async -> (any TunnelProtocol)? {
        await tunnelManager.tunnelStore.getPersistentTunnel()
    }

    func createNewTunnel() async -> any TunnelProtocol {
        await tunnelManager.tunnelStore.createNewTunnel()
    }

    func setTunnel(_ tunnel: (any TunnelProtocol)?, shouldRefreshTunnelState: Bool) async {
        await tunnelManager.setTunnel(tunnel, shouldRefreshTunnelState: shouldRefreshTunnelState)
    }

    func getTunnelStatus() async -> TunnelStatus {
        await tunnelManager._tunnelStatus
    }

    func updateTunnelStatus(_ block: @Sendable (inout TunnelStatus) -> Void) async -> TunnelStatus {
        await tunnelManager.setTunnelStatus(block)
    }

    var isConfigurationLoaded: Bool {
        tunnelManager.isConfigurationLoaded
    }

    var settings: LatestTunnelSettings {
        tunnelManager.settings
    }

    func getDeviceState() async -> DeviceState {
        await tunnelManager._deviceState
    }

    func setConfigurationLoaded() async {
        await tunnelManager.setConfigurationLoaded()
    }

    func setSettings(_ settings: LatestTunnelSettings, persist: Bool) async {
        await tunnelManager.setSettings(settings, persist: persist)
    }

    func setDeviceState(_ deviceState: DeviceState, persist: Bool) async {
        await tunnelManager.setDeviceState(deviceState, persist: persist)
    }

    func removeLastUsedAccount() async {
        await tunnelManager.removeLastUsedAccount()
    }

    func setLastUsedAccount(_ accountNumber: String) async {
        await tunnelManager.setLastUsedAccount(accountNumber)
    }

    func unsetTunnelConfiguration() async {
        await tunnelManager.unsetTunnelConfiguration()
    }

    func startTunnel() async {
        await tunnelManager.startTunnel()
    }

    func prepareForVPNConfigurationDeletion() async {
        await tunnelManager.prepareForVPNConfigurationDeletion()
    }

    func selectRelays() async throws -> SelectedRelays {
        try await tunnelManager.selectRelays()
    }

    func handleRestError(_ error: Error) async {
        await tunnelManager.handleRestError(error)
    }
}

// MARK: - RelayCacheTrackerObserver

#if NOT_PRODUCTION_PERMANENT
    extension TunnelManager: RelayCacheTrackerObserver {
        nonisolated func relayCacheTracker(
            _ tracker: RelayCacheTracker,
            didUpdateCachedRelays cachedRelays: CachedRelays
        ) {
            // Only reconnect if relays are now available
            guard !cachedRelays.isEmpty else { return }

            Task { [weak self] in
                guard let self else { return }
                // If tunnel is active, trigger reconnect to re-evaluate with new relays
                if self._tunnelStatus.state.isSecured {
                    self.reconnectTunnel(selectNewRelay: false)
                }
            }
        }
    }
#endif
