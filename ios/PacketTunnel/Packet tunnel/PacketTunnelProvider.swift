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
import MullvadSettings
import MullvadTransport
import MullvadTypes
import Network
import NetworkExtension
import Operations
import PacketTunnelCore
import RelayCache
import RelaySelector
import WireGuardKit

/// Restart interval (in seconds) for the tunnel that failed to start early on.
private let tunnelStartupFailureRestartInterval: Duration = .seconds(2)

/// Delay before trying to reconnect tunnel after private key rotation.
private let keyRotationTunnelReconnectionDelay: Duration = .minutes(2)

class PacketTunnelProvider: NEPacketTunnelProvider {
    /// Tunnel provider logger.
    private let providerLogger: Logger

    /// WireGuard adapter logger.
    private let tunnelLogger: Logger

    /// Internal queue.
    private let dispatchQueue = DispatchQueue(label: "PacketTunnel", qos: .utility)

    /// WireGuard adapter.
    private var adapter: WireGuardAdapter!

    /// Current tunnel provider state.
    private var state: State = .stopped

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

    private let constraintsUpdater = RelayConstraintsUpdater()

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
        loggerBuilder.addFileOutput(fileURL: ApplicationConfiguration.logFileURL(for: .packetTunnel))

        #if DEBUG
        loggerBuilder.addOSLogOutput(subsystem: ApplicationTarget.packetTunnel.bundleIdentifier)
        #endif

        loggerBuilder.install()

        providerLogger = Logger(label: "PacketTunnelProvider")
        tunnelLogger = Logger(label: "WireGuard")

        let containerURL = ApplicationConfiguration.containerURL
        let addressCache = REST.AddressCache(canWriteToCache: false, cacheDirectory: containerURL)
        addressCache.loadFromFile()

        relayCache = RelayCache(cacheDirectory: containerURL)

        let urlSession = REST.makeURLSession()
        let urlSessionTransport = URLSessionTransport(urlSession: urlSession)
        let shadowsocksCache = ShadowsocksConfigurationCache(cacheDirectory: containerURL)
        let transportProvider = TransportProvider(
            urlSessionTransport: urlSessionTransport,
            relayCache: relayCache,
            addressCache: addressCache,
            shadowsocksCache: shadowsocksCache,
            constraintsUpdater: constraintsUpdater
        )

        let proxyFactory = REST.ProxyFactory.makeProxyFactory(
            transportProvider: transportProvider,
            addressCache: addressCache
        )

        urlRequestProxy = URLRequestProxy(
            dispatchQueue: dispatchQueue,
            transportProvider: transportProvider
        )
        accountsProxy = proxyFactory.createAccountsProxy()
        devicesProxy = proxyFactory.createDevicesProxy()

        super.init()

        let wgAdapter = WgAdapter(packetTunnelProvider: self)
        adapter = wgAdapter.adapter

        tunnelMonitor = createTunnelMonitor(wireGuardAdapter: adapter)
        tunnelMonitor.onEvent = { [weak self] event in
            self?.handleTunnelMonitorEvent(event)
        }
    }

    override func startTunnel(options: [String: NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        dispatchQueue.async {
            guard self.state.canTransition(to: .starting) else { return }

            self.state = .starting(completionHandler: completionHandler)

            // Parse relay selector from tunnel options.
            let parsedOptions = self.parseStartOptions(options ?? [:])
            self.providerLogger.debug("\(parsedOptions.logFormat())")

            // Read tunnel configuration.
            let tunnelConfiguration: PacketTunnelConfiguration
            do {
                let initialRelay: NextRelay = parsedOptions.selectorResult.map { .set($0) } ?? .automatic

                tunnelConfiguration = try self.makeConfiguration(initialRelay)
            } catch {
                self.providerLogger.error(
                    error: error,
                    message: "Failed to read tunnel configuration when starting the tunnel."
                )

                self.configurationError = error

                self.startEmptyTunnel()
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

                        if case let .starting(completionHandler) = self.state {
                            completionHandler(error)
                            self.state = .stopped
                        }
                    } else {
                        self.providerLogger.debug("Started the tunnel.")

                        self.tunnelAdapterDidStart()

                        self.tunnelMonitor.start(
                            probeAddress: tunnelConfiguration.selectorResult.endpoint.ipv4Gateway
                        )
                    }
                }
            }
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        dispatchQueue.async {
            guard self.state.canTransition(to: .stopping) else { return }

            self.providerLogger.debug("Stop the tunnel: \(reason)")

            self.state = .stopping(completionHandler: completionHandler)

            self.cancelTunnelReconnectionTimer()
            self.cancelTunnelStartupFailureRecovery()

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

                        if case let .stopping(completionHandler) = self.state {
                            completionHandler()
                            self.state = .stopped
                        }
                    }
                }
            }
        }
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        dispatchQueue.async {
            // Do not handle app messages if they are received after the tunnel began shutdown procedure.
            guard self.state.primitiveState == .starting || self.state.primitiveState == .started else { return }

            let message: TunnelProviderMessage
            do {
                message = try TunnelProviderMessage(messageData: messageData)
            } catch {
                self.providerLogger.error(error: error, message: "Failed to decode the app message.")

                completionHandler?(nil)
                return
            }

            self.providerLogger.trace("Received app message: \(message)")

            switch message {
            case let .reconnectTunnel(appSelectorResult):
                self.providerLogger.debug("Reconnecting the tunnel...")

                let nextRelay: NextRelay = (appSelectorResult ?? self.selectorResult).map { .set($0) } ?? .automatic
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
        tunnelMonitor.onSleep()
        completionHandler()
    }

    override func wake() {
        tunnelMonitor.onWake()
    }

    // MARK: - Private: Tunnel monitoring

    private func handleTunnelMonitorEvent(_ event: TunnelMonitorEvent) {
        switch event {
        case .connectionEstablished:
            tunnelConnectionEstablished()

        case .connectionLost:
            tunnelConnectionLost()

        case let .networkReachabilityChanged(isReachable):
            tunnelReachabilityChanged(isReachable)
        }
    }

    private func tunnelConnectionEstablished() {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        providerLogger.debug("Connection established.")

        if case let .starting(completionHandler) = state {
            completionHandler(nil)
            state = .started
        }

        numberOfFailedAttempts = 0

        checkDeviceStateTask?.cancel()
        checkDeviceStateTask = nil

        setReconnecting(false)
    }

    private func tunnelConnectionLost() {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        let (value, isOverflow) = numberOfFailedAttempts.addingReportingOverflow(1)
        numberOfFailedAttempts = isOverflow ? 0 : value

        if numberOfFailedAttempts.isMultiple(of: 2) {
            startDeviceCheck()
        }

        providerLogger.debug("Recover connection. Picking next relay...")

        reconnectTunnel(to: .automatic, shouldStopTunnelMonitor: false)
    }

    private func tunnelReachabilityChanged(_ isNetworkReachable: Bool) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        guard self.isNetworkReachable != isNetworkReachable else { return }

        self.isNetworkReachable = isNetworkReachable

        // Switch tunnel into reconnecting state when offline.
        if !isNetworkReachable {
            setReconnecting(true)
        }
    }

    // MARK: - Private

    private func createTunnelMonitor(wireGuardAdapter: WgAdapter) -> TunnelMonitor {
        TunnelMonitor(
            eventQueue: dispatchQueue,
            pinger: Pinger(replyQueue: dispatchQueue),
            tunnelDeviceInfo: wgAdapter,
            defaultPathObserver: PacketTunnelPathObserver(packetTunnelProvider: self),
            timings: TunnelMonitorTimings()
        )
    }

    private func startTunnelReconnectionTimer(reconnectionDelay: Duration) {
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

        timer.schedule(wallDeadline: .now() + reconnectionDelay)
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
            reconnectTunnel(to: .automatic, shouldStopTunnelMonitor: false) { [weak self] error in
                if error == nil {
                    self?.cancelTunnelStartupFailureRecovery()
                }
            }
        }

        timer.schedule(
            wallDeadline: .now() + tunnelStartupFailureRestartInterval,
            repeating: tunnelStartupFailureRestartInterval.timeInterval
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

    /**
     Called once the tunnel was able to read configuration and start WireGuard adapter.
     */
    private func tunnelAdapterDidStart() {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        startDeviceCheck(shouldImmediatelyRotateKeyOnMismatch: true)
    }

    private func startEmptyTunnel() {
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

                    if case let .starting(completionHandler) = self.state {
                        completionHandler(error)
                        self.state = .stopped
                    }
                } else {
                    self.providerLogger.debug("Started an empty tunnel.")

                    self.tunnelAdapterDidStart()
                }
            }
        }
    }

    private func setReconnecting(_ reconnecting: Bool) {
        // Raise reasserting flag, but only if tunnel has already moved to connected state once.
        // Otherwise keep the app in connecting state until it manages to establish the very first
        // connection.
        if case .started = state {
            reasserting = reconnecting
        }
    }

    private func parseStartOptions(_ options: [String: NSObject]) -> StartOptions {
        let tunnelOptions = PacketTunnelOptions(rawOptions: options)
        var parsedOptions = StartOptions(launchSource: tunnelOptions.isOnDemand() ? .onDemand : .app)

        do {
            if let selectorResult = try tunnelOptions.getSelectorResult() {
                parsedOptions.launchSource = .app
                parsedOptions.selectorResult = selectorResult
            } else {
                parsedOptions.launchSource = tunnelOptions.isOnDemand() ? .onDemand : .system
            }
        } catch {
            providerLogger.error(error: error, message: "Failed to decode relay selector result passed from the app.")
        }

        return parsedOptions
    }

    private func makeConfiguration(_ nextRelay: NextRelay) throws -> PacketTunnelConfiguration {
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

        constraintsUpdater.onNewConstraints?(tunnelSettings.relayConstraints)

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
        guard state.primitiveState == .starting || state.primitiveState == .started else { return }

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

    private func reconnectTunnelInner(to nextRelay: NextRelay, completionHandler: ((Error?) -> Void)? = nil) {
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
        -> RelaySelectorResult {
        let cachedRelayList = try relayCache.read()

        return try RelaySelector.evaluate(
            relays: cachedRelayList.relays,
            constraints: relayConstraints,
            numberOfFailedAttempts: packetTunnelStatus.numberOfFailedAttempts
        )
    }

    // MARK: - Device check

    /**
     Start device diagnostics to determine the reason why the tunnel is not functional.

     This involves the following steps:

     1. Fetch account and device data.
     2. Check account validity and whether it has enough time left.
     3. Verify that current device is registered with backend and that both device and backend point to the same public
        key.
     4. Rotate WireGuard key on key mismatch.
     */
    private func startDeviceCheck(shouldImmediatelyRotateKeyOnMismatch: Bool = false) {
        let checkOperation = DeviceCheckOperation(
            dispatchQueue: dispatchQueue,
            remoteSevice: DeviceCheckRemoteService(accountsProxy: accountsProxy, devicesProxy: devicesProxy),
            deviceStateAccessor: DeviceStateAccessor(),
            rotateImmediatelyOnKeyMismatch: shouldImmediatelyRotateKeyOnMismatch
        ) { [self] result in
            guard var newDeviceCheck = result.value else { return }

            if newDeviceCheck.accountVerdict == .invalid || newDeviceCheck.deviceVerdict == .revoked {
                // Stop tunnel monitor when device is revoked or account is invalid.
                tunnelMonitor.stop()
            } else if case .succeeded = newDeviceCheck.keyRotationStatus {
                // Tell the tunnel to reconnect using new private key if key was rotated dring device check.
                reconnectTunnel(to: .automatic, shouldStopTunnelMonitor: false)
            }

            // Retain the last key rotation status that isn't `.noAction` so that UI could keep track of when rotation
            // attempts take place which should give it a hint when to refresh device state from settings.
            if let deviceCheck, newDeviceCheck.keyRotationStatus == .noAction {
                newDeviceCheck.keyRotationStatus = deviceCheck.keyRotationStatus
            }

            deviceCheck = newDeviceCheck
        }

        operationQueue.addOperation(checkOperation)
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

    // swiftlint:disable:next file_length
}

/// Enum describing the next relay to connect to.
public enum NextRelay {
    /// Connect to pre-selected relay.
    case set(RelaySelectorResult)

    /// Determine next relay using relay selector.
    case automatic
}
