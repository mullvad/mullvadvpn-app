//
//  PacketTunnelProvider.swift
//  PacketTunnel
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTransport
import MullvadTypes
import Network
import NetworkExtension
import Operations
import RelayCache
import RelaySelector
import TunnelProviderMessaging
import WireGuardKit

/// Restart interval (in seconds) for the tunnel that failed to start early on.
private let tunnelStartupFailureRestartInterval: TimeInterval = 2

/// Delay before trying to reconnect tunnel after private key rotation.
private let keyRotationTunnelReconnectionDelay = 60 * 2

class PacketTunnelProvider: NEPacketTunnelProvider, TunnelMonitorDelegate {
    /// Tunnel provider logger.
    private let providerLogger: Logger

    /// WireGuard adapter logger.
    private let tunnelLogger: Logger

    /// Internal queue.
    private let dispatchQueue = DispatchQueue(label: "PacketTunnel", qos: .utility)

    /// WireGuard adapter.
    private var adapter: WireGuardAdapter!

    /// Raised once tunnel establishes connection in the very first time, before calling the system
    /// completion handler passed into `startTunnel`.
    private var isConnected = false

    /// Raised once tunnel receives the first call to `stopTunnel()`.
    /// Once this happens all requests to reconnect the tunnel will be ignored.
    private var isStopping = false

    /// Flag indicating whether network is reachable.
    private var isNetworkReachable = true

    /// Struct holding result of the last device check.
    private var deviceCheck: DeviceCheck?

    /// Number of consecutive connection failure attempts.
    private var numberOfFailedAttempts: UInt = 0

    /// Last wireguard error.
    private var wgError: WireGuardAdapterError?

    /// Last configuration read error.
    private var configurationError: Error?

    /// Repeating timer used for restarting the tunnel if it had failed during the startup sequence.
    private var tunnelStartupFailureRecoveryTimer: DispatchSourceTimer?

    /// Relay cache.
    private let relayCache: RelayCache

    /// Current selector result.
    private var selectorResult: RelaySelectorResult?

    /// A system completion handler passed from startTunnel and saved for later use once the
    /// connection is established.
    private var startTunnelCompletionHandler: (() -> Void)?

    /// Tunnel monitor.
    private var tunnelMonitor: TunnelMonitor!

    /// Request proxy used to perform URLRequests bypassing VPN.
    private let urlRequestProxy: URLRequestProxy

    /// Account data request proxy
    private let accountsProxy: REST.AccountsProxy

    /// Device data request proxy
    private let devicesProxy: REST.DevicesProxy

    /// Last device check task.
    private var checkDeviceStateTask: Cancellable?

    /// Last task to reconnect the tunnel.
    private var reconnectTunnelTask: Operation?

    /// Internal operation queue.
    private let operationQueue = AsyncOperationQueue()

    /// Timer for tunnel reconnection. Used to delay reconnection when a private key has just been
    /// rotated, to account for latency in key propagation to relays.
    private var tunnelReconnectionTimer: DispatchSourceTimer?

    /// Current device state for the tunnel.
    private var cachedDeviceState: DeviceState?

    /// Whether to use the cached device state.
    private var useCachedDeviceState = false

    /// Returns `PacketTunnelStatus` used for sharing with main bundle process.
    private var packetTunnelStatus: PacketTunnelStatus {
        let errors: [PacketTunnelErrorWrapper?] = [
            wgError.flatMap { PacketTunnelErrorWrapper(error: $0) },
            configurationError.flatMap { PacketTunnelErrorWrapper(error: $0) },
        ]

        return PacketTunnelStatus(
            lastErrors: errors.compactMap { $0 },
            isNetworkReachable: isNetworkReachable,
            deviceCheck: deviceCheck,
            tunnelRelay: selectorResult?.packetTunnelRelay,
            numberOfFailedAttempts: numberOfFailedAttempts
        )
    }

    override init() {
        var loggerBuilder = LoggerBuilder()

        let pid = ProcessInfo.processInfo.processIdentifier
        loggerBuilder.metadata["pid"] = .string("\(pid)")

        let bundleIdentifier = Bundle.main.bundleIdentifier!

        try? loggerBuilder.addFileOutput(
            securityGroupIdentifier: ApplicationConfiguration.securityGroupIdentifier,
            basename: bundleIdentifier
        )

        #if DEBUG
        loggerBuilder.addOSLogOutput(subsystem: bundleIdentifier)
        #endif

        loggerBuilder.install()

        providerLogger = Logger(label: "PacketTunnelProvider")
        tunnelLogger = Logger(label: "WireGuard")

        let containerURL = FileManager.default
            .containerURL(forSecurityApplicationGroupIdentifier: ApplicationConfiguration.securityGroupIdentifier)!
        let addressCache = REST.AddressCache(
            canWriteToCache: false, cacheFolder: containerURL
        )

        addressCache.initCache()

        relayCache = RelayCache(cacheFolder: containerURL)

        let urlSession = REST.makeURLSession()
        let urlSessionTransport = URLSessionTransport(urlSession: urlSession)
        let shadowsocksCache = ShadowsocksConfigurationCache(cacheFolder: containerURL)
        let transportProvider = TransportProvider(
            urlSessionTransport: urlSessionTransport,
            relayCache: relayCache,
            addressCache: addressCache,
            shadowsocksCache: shadowsocksCache
        )

        let proxyFactory = REST.ProxyFactory.makeProxyFactory(
            transportProvider: { transportProvider },
            addressCache: addressCache
        )

        urlRequestProxy = URLRequestProxy(
            dispatchQueue: dispatchQueue,
            transportProvider: { transportProvider }
        )
        accountsProxy = proxyFactory.createAccountsProxy()
        devicesProxy = proxyFactory.createDevicesProxy()

        super.init()

        adapter = WireGuardAdapter(
            with: self,
            shouldHandleReasserting: false,
            logHandler: { [weak self] logLevel, message in
                self?.dispatchQueue.async {
                    self?.tunnelLogger.log(level: logLevel.loggerLevel, "\(message)")
                }
            }
        )

        tunnelMonitor = TunnelMonitor(
            delegateQueue: dispatchQueue,
            packetTunnelProvider: self,
            adapter: adapter
        )
        tunnelMonitor.delegate = self
    }

    override func startTunnel(
        options: [String: NSObject]?,
        completionHandler: @escaping (Error?) -> Void
    ) {
        dispatchQueue.async {
            let tunnelOptions = PacketTunnelOptions(rawOptions: options ?? [:])
            var appSelectorResult: RelaySelectorResult?

            // Parse relay selector from tunnel options.
            do {
                appSelectorResult = try tunnelOptions.getSelectorResult()

                switch appSelectorResult {
                case let .some(selectorResult):
                    self.providerLogger.debug(
                        "Start the tunnel via app, connect to \(selectorResult.relay.hostname)."
                    )

                case .none:
                    if tunnelOptions.isOnDemand() {
                        self.providerLogger.debug("Start the tunnel via on-demand rule.")
                    } else {
                        self.providerLogger.debug("Start the tunnel via system.")
                    }
                }
            } catch {
                self.providerLogger.debug("Start the tunnel via app.")
                self.providerLogger.error(
                    error: error,
                    message: """
                    Failed to decode relay selector result passed from the app. \
                    Will continue by picking new relay.
                    """
                )
            }

            // Read tunnel configuration.
            let tunnelConfiguration: PacketTunnelConfiguration
            do {
                let initialRelay: NextRelay = appSelectorResult.map { .set($0) } ?? .automatic

                tunnelConfiguration = try self.makeConfiguration(initialRelay)
            } catch {
                self.providerLogger.error(
                    error: error,
                    message: "Failed to read tunnel configuration when starting the tunnel."
                )

                self.configurationError = error

                self.startEmptyTunnel(completionHandler: completionHandler)
                self.beginTunnelStartupFailureRecovery()
                return
            }

            // Set tunnel status.
            let selectorResult = tunnelConfiguration.selectorResult
            self.selectorResult = selectorResult
            self.providerLogger.debug("Set tunnel relay to \(selectorResult.relay.hostname).")

            // Start tunnel.
            self.adapter.start(tunnelConfiguration: tunnelConfiguration.wgTunnelConfig) { error in
                self.dispatchQueue.async {
                    if let error {
                        self.providerLogger.error(
                            error: error,
                            message: "Failed to start the tunnel."
                        )

                        completionHandler(error)
                    } else {
                        self.providerLogger.debug("Started the tunnel.")

                        self.startTunnelCompletionHandler = { [weak self] in
                            self?.isConnected = true
                            completionHandler(nil)
                        }

                        self.tunnelMonitor.start(
                            probeAddress: tunnelConfiguration.selectorResult.endpoint.ipv4Gateway
                        )
                    }
                }
            }
        }
    }

    override func stopTunnel(
        with reason: NEProviderStopReason,
        completionHandler: @escaping () -> Void
    ) {
        dispatchQueue.async {
            self.providerLogger.debug("Stop the tunnel: \(reason)")

            self.isStopping = true
            self.cancelTunnelReconnectionTimer()
            self.cancelTunnelStartupFailureRecovery()
            self.startTunnelCompletionHandler = nil

            // Cancel all operations: reconnection requests, network requests.
            self.operationQueue.cancelAllOperations()

            // Stop tunnel monitor after all operations are kicked off the queue.
            self.operationQueue.addBarrierBlock {
                self.tunnelMonitor.stop()

                self.adapter.stop { error in
                    self.dispatchQueue.async {
                        if let error {
                            self.providerLogger.error(
                                error: error,
                                message: "Failed to stop the tunnel gracefully."
                            )
                        } else {
                            self.providerLogger.debug("Stopped the tunnel.")
                        }
                        completionHandler()
                    }
                }
            }
        }
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        dispatchQueue.async {
            let message: TunnelProviderMessage
            do {
                message = try TunnelProviderMessage(messageData: messageData)
            } catch {
                self.providerLogger.error(
                    error: error,
                    message: "Failed to decode the app message."
                )

                completionHandler?(nil)
                return
            }

            self.providerLogger.trace("Received app message: \(message)")

            switch message {
            case let .reconnectTunnel(appSelectorResult):
                self.providerLogger.debug("Reconnecting the tunnel...")

                let nextRelay: NextRelay = (appSelectorResult ?? self.selectorResult)
                    .map { .set($0) } ?? .automatic

                self.reconnectTunnel(to: nextRelay, shouldStopTunnelMonitor: true)

                completionHandler?(nil)

            case .getTunnelStatus:
                var response: Data?
                do {
                    response = try TunnelProviderReply(self.packetTunnelStatus).encode()
                } catch {
                    self.providerLogger.error(
                        error: error,
                        message: "Failed to encode tunnel status reply."
                    )
                }

                completionHandler?(response)

            case let .sendURLRequest(proxyRequest):
                self.urlRequestProxy.sendRequest(proxyRequest) { response in
                    var reply: Data?
                    do {
                        reply = try TunnelProviderReply(response).encode()
                    } catch {
                        self.providerLogger.error(
                            error: error,
                            message: "Failed to encode ProxyURLResponse."
                        )
                    }
                    completionHandler?(reply)
                }

            case let .cancelURLRequest(id):
                self.urlRequestProxy.cancelRequest(identifier: id)
                completionHandler?(nil)

            case .privateKeyRotation:
                self.startTunnelReconnectionTimer(
                    reconnectionDelay: keyRotationTunnelReconnectionDelay
                )
                completionHandler?(nil)
            }
        }
    }

    override func sleep(completionHandler: @escaping () -> Void) {
        tunnelMonitor.onSleep {
            completionHandler()
        }
    }

    override func wake() {
        tunnelMonitor.onWake()
    }

    // MARK: - TunnelMonitorDelegate

    func tunnelMonitorDidDetermineConnectionEstablished(_ tunnelMonitor: TunnelMonitor) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        providerLogger.debug("Connection established.")

        startTunnelCompletionHandler?()
        startTunnelCompletionHandler = nil

        numberOfFailedAttempts = 0

        checkDeviceStateTask?.cancel()
        checkDeviceStateTask = nil

        deviceCheck = nil

        setReconnecting(false)
    }

    func tunnelMonitorDelegateShouldHandleConnectionRecovery(_ tunnelMonitor: TunnelMonitor) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        let (value, isOverflow) = numberOfFailedAttempts.addingReportingOverflow(1)
        numberOfFailedAttempts = isOverflow ? 0 : value

        if numberOfFailedAttempts.isMultiple(of: 2) {
            startDeviceCheck()
        }

        providerLogger.debug("Recover connection. Picking next relay...")

        reconnectTunnel(to: .automatic, shouldStopTunnelMonitor: false)
    }

    func tunnelMonitor(
        _ tunnelMonitor: TunnelMonitor,
        networkReachabilityStatusDidChange isNetworkReachable: Bool
    ) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        guard self.isNetworkReachable != isNetworkReachable else { return }

        self.isNetworkReachable = isNetworkReachable

        // Switch tunnel into reconnecting state when offline.
        if !isNetworkReachable {
            setReconnecting(true)
        }
    }

    // MARK: - Private

    private func startTunnelReconnectionTimer(reconnectionDelay: Int) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        providerLogger.debug("Delaying tunnel reconnection by \(reconnectionDelay) seconds...")
        useCachedDeviceState = true

        let timer = DispatchSource.makeTimerSource(queue: dispatchQueue)

        timer.setEventHandler { [weak self] in
            self?.providerLogger.debug("Reconnecting the tunnel...")

            let nextRelay: NextRelay = self?.selectorResult
                .map { .set($0) } ?? .automatic

            self?.useCachedDeviceState = false
            self?.reconnectTunnel(to: nextRelay, shouldStopTunnelMonitor: true)
        }

        timer.setCancelHandler { [weak self] in
            self?.useCachedDeviceState = false
        }

        timer.schedule(deadline: .now() + .seconds(reconnectionDelay))
        timer.activate()

        tunnelReconnectionTimer?.cancel()
        tunnelReconnectionTimer = timer
    }

    private func cancelTunnelReconnectionTimer() {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        tunnelReconnectionTimer?.cancel()
        tunnelReconnectionTimer = nil
    }

    private func beginTunnelStartupFailureRecovery() {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        let timer = DispatchSource.makeTimerSource(queue: dispatchQueue)
        timer.setEventHandler { [weak self] in
            guard let self else { return }

            providerLogger.debug("Restart the tunnel that had startup failure.")
            reconnectTunnel(
                to: .automatic,
                shouldStopTunnelMonitor: false
            ) { [weak self] error in
                if error == nil {
                    self?.cancelTunnelStartupFailureRecovery()
                }
            }
        }

        timer.schedule(
            wallDeadline: .now() + tunnelStartupFailureRestartInterval,
            repeating: tunnelStartupFailureRestartInterval
        )
        timer.activate()

        tunnelStartupFailureRecoveryTimer?.cancel()
        tunnelStartupFailureRecoveryTimer = timer
    }

    private func cancelTunnelStartupFailureRecovery() {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        tunnelStartupFailureRecoveryTimer?.cancel()
        tunnelStartupFailureRecoveryTimer = nil
    }

    private func startEmptyTunnel(completionHandler: @escaping (Error?) -> Void) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        let emptyTunnelConfiguration = TunnelConfiguration(
            name: nil,
            interface: InterfaceConfiguration(privateKey: PrivateKey()),
            peers: []
        )

        adapter.start(tunnelConfiguration: emptyTunnelConfiguration) { error in
            self.dispatchQueue.async {
                if let error {
                    self.providerLogger.error(
                        error: error,
                        message: "Failed to start an empty tunnel."
                    )

                    completionHandler(error)
                } else {
                    self.providerLogger.debug("Started an empty tunnel.")

                    self.startTunnelCompletionHandler = { [weak self] in
                        self?.isConnected = true
                        completionHandler(nil)
                    }
                }
            }
        }
    }

    private func setReconnecting(_ reconnecting: Bool) {
        // Raise reasserting flag, but only if tunnel has already moved to connected state once.
        // Otherwise keep the app in connecting state until it manages to establish the very first
        // connection.
        if isConnected {
            reasserting = reconnecting
        }
    }

    private func makeConfiguration(_ nextRelay: NextRelay)
        throws -> PacketTunnelConfiguration
    {
        let tunnelSettings = try SettingsManager.readSettings()
        let selectorResult: RelaySelectorResult

        var deviceState: DeviceState
        if let cachedDeviceState, useCachedDeviceState {
            deviceState = cachedDeviceState
        } else {
            deviceState = try SettingsManager.readDeviceState()
            cachedDeviceState = deviceState
        }

        switch nextRelay {
        case .automatic:
            selectorResult = try selectRelayEndpoint(
                relayConstraints: tunnelSettings.relayConstraints
            )
        case let .set(aSelectorResult):
            selectorResult = aSelectorResult
        }

        return PacketTunnelConfiguration(
            deviceState: deviceState,
            tunnelSettings: tunnelSettings,
            selectorResult: selectorResult
        )
    }

    private func reconnectTunnel(
        to nextRelay: NextRelay,
        shouldStopTunnelMonitor: Bool,
        completionHandler: ((Error?) -> Void)? = nil
    ) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        // Ignore all requests to reconnect once tunnel is preparing to stop.
        guard !isStopping else { return }

        let blockOperation = AsyncBlockOperation(dispatchQueue: dispatchQueue, block: { finish in
            if shouldStopTunnelMonitor {
                self.tunnelMonitor.stop()
            }

            self.reconnectTunnelInner(to: nextRelay) { error in
                completionHandler?(error)
                finish(nil)
            }
        })

        if let reconnectTunnelTask {
            blockOperation.addDependency(reconnectTunnelTask)
        }

        reconnectTunnelTask?.cancel()
        reconnectTunnelTask = blockOperation

        operationQueue.addOperation(blockOperation)
    }

    private func reconnectTunnelInner(
        to nextRelay: NextRelay,
        completionHandler: ((Error?) -> Void)? = nil
    ) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        // Read tunnel configuration.
        let tunnelConfiguration: PacketTunnelConfiguration
        do {
            tunnelConfiguration = try makeConfiguration(nextRelay)
            configurationError = nil
        } catch {
            providerLogger.error(
                error: error,
                message: "Failed to produce new configuration."
            )

            configurationError = error

            completionHandler?(error)
            return
        }

        // Copy old relay.
        let oldSelectorResult = selectorResult
        let newTunnelRelay = tunnelConfiguration.selectorResult.packetTunnelRelay

        // Update tunnel status.
        selectorResult = tunnelConfiguration.selectorResult

        providerLogger.debug("Set tunnel relay to \(newTunnelRelay.hostname).")
        setReconnecting(true)

        adapter.update(tunnelConfiguration: tunnelConfiguration.wgTunnelConfig) { error in
            self.dispatchQueue.async {
                if let error {
                    self.wgError = error
                    self.providerLogger.error(
                        error: error,
                        message: "Failed to update WireGuard configuration."
                    )

                    // Revert to previously used relay selector as it's very likely that we keep
                    // using previous configuration.
                    self.selectorResult = oldSelectorResult
                    self.providerLogger.debug(
                        "Reset tunnel relay to \(oldSelectorResult?.relay.hostname ?? "none")."
                    )
                    self.setReconnecting(false)
                } else {
                    self.tunnelMonitor.start(
                        probeAddress: tunnelConfiguration.selectorResult.endpoint.ipv4Gateway
                    )
                }
                completionHandler?(error)
            }
        }
    }

    /// Load relay cache with potential networking to refresh the cache and pick the relay for the
    /// given relay constraints.
    private func selectRelayEndpoint(relayConstraints: RelayConstraints) throws
        -> RelaySelectorResult
    {
        let cachedRelayList = try relayCache.read()

        return try RelaySelector.evaluate(
            relays: cachedRelayList.relays,
            constraints: relayConstraints,
            numberOfFailedAttempts: packetTunnelStatus.numberOfFailedAttempts
        )
    }

    // MARK: - Device check

    /// Fetch account and device data to verify account expiry and device status.
    /// Saves the result into deviceCheck
    private func startDeviceCheck() {
        let deviceState: DeviceState
        do {
            deviceState = try SettingsManager.readDeviceState()
        } catch {
            providerLogger.error(
                error: error,
                message: "Failed to read device state."
            )
            return
        }

        guard case let .loggedIn(storedAccountData, storedDeviceData) = deviceState else {
            return
        }

        providerLogger.debug("Start device check.")

        let accountOperation = createGetAccountDataOperation(
            accountNumber: storedAccountData.number
        )
        let deviceOperation = createGetDeviceDataOperation(
            accountNumber: storedAccountData.number,
            identifier: storedDeviceData.identifier
        )

        let completionOperation = AsyncBlockOperation(dispatchQueue: dispatchQueue) { [weak self] in
            guard let self else { return }

            var newAccountExpiry: Date?
            var newDeviceRevoked: Bool?

            switch accountOperation.result {
            case let .failure(error):
                if !error.isOperationCancellationError {
                    providerLogger.error(
                        error: error,
                        message: "Failed to fetch account data."
                    )
                }

            case let .success(accountData):
                newAccountExpiry = accountData.expiry

            case .none:
                break
            }

            switch deviceOperation.result {
            case let .failure(error):
                if let restError = error as? REST.Error,
                   restError.compareErrorCode(.deviceNotFound)
                {
                    newDeviceRevoked = true
                } else if !error.isOperationCancellationError {
                    providerLogger.error(
                        error: error,
                        message: "Failed to fetch device data."
                    )
                }

            case .none, .success:
                break
            }

            if var deviceCheck {
                deviceCheck.update(
                    accountExpiry: newAccountExpiry,
                    isDeviceRevoked: newDeviceRevoked
                )

                self.deviceCheck = deviceCheck
            } else {
                deviceCheck = DeviceCheck(
                    identifier: UUID(),
                    isDeviceRevoked: newDeviceRevoked,
                    accountExpiry: newAccountExpiry
                )
            }

            if newDeviceRevoked ?? false {
                tunnelMonitor.stop()
            }
        }

        completionOperation.addDependencies([accountOperation, deviceOperation])

        let groupOperation = GroupOperation(operations: [
            accountOperation,
            deviceOperation,
            completionOperation,
        ])
        checkDeviceStateTask = groupOperation

        operationQueue.addOperation(groupOperation)
    }

    private func createGetAccountDataOperation(accountNumber: String) -> ResultOperation<REST.AccountData> {
        return ResultBlockOperation<REST.AccountData>(dispatchQueue: dispatchQueue) { finish -> Cancellable in
            return self.accountsProxy.getAccountData(
                accountNumber: accountNumber,
                retryStrategy: .noRetry,
                completion: finish
            )
        }
    }

    private func createGetDeviceDataOperation(
        accountNumber: String,
        identifier: String
    ) -> ResultOperation<REST.Device> {
        return ResultBlockOperation<REST.Device>(dispatchQueue: dispatchQueue) { finish -> Cancellable in
            return self.devicesProxy.getDevice(
                accountNumber: accountNumber,
                identifier: identifier,
                retryStrategy: .noRetry,
                completion: finish
            )
        }
    }
}

/// Enum describing the next relay to connect to.
private enum NextRelay {
    /// Connect to pre-selected relay.
    case set(RelaySelectorResult)

    /// Determine next relay using relay selector.
    case automatic
}

extension DeviceCheck {
    mutating func update(accountExpiry: Date?, isDeviceRevoked: Bool?) {
        var shouldChangeIdentifier = false

        if let accountExpiry, self.accountExpiry != accountExpiry {
            shouldChangeIdentifier = true
            self.accountExpiry = accountExpiry
        }

        if let isDeviceRevoked, self.isDeviceRevoked != isDeviceRevoked {
            shouldChangeIdentifier = true
            self.isDeviceRevoked = isDeviceRevoked
        }

        if shouldChangeIdentifier {
            identifier = UUID()
        }
    }
}

extension PacketTunnelErrorWrapper {
    init?(error: Error) {
        switch error {
        case let error as WireGuardAdapterError:
            self = .wireguard(error.localizedDescription)

        case is UnsupportedSettingsVersionError:
            self = .configuration(.outdatedSchema)

        case let keychainError as KeychainError where keychainError == .interactionNotAllowed:
            self = .configuration(.deviceLocked)

        case let error as ReadSettingsVersionError:
            if case KeychainError.interactionNotAllowed = error.underlyingError as? KeychainError {
                self = .configuration(.deviceLocked)
            } else {
                self = .configuration(.readFailure)
            }

        case is NoRelaysSatisfyingConstraintsError:
            self = .configuration(.noRelaysSatisfyingConstraints)

        default:
            return nil
        }
    }
}
