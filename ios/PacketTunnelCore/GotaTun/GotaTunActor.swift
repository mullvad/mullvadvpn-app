//
//  GotaTunActor.swift
//  PacketTunnelCore
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import Network

/// Events processed by the actor's internal loop.
private enum GotaTunEvent: Sendable {
    case start(StartOptions)
    case stop
    case adapterConnected
    case adapterTimeout
    case adapterError(GotaTunError)
    case reconnect(NextRelays, ActorReconnectReason)
    case networkReachability(NWPath.Status)
    case setErrorState(BlockedStateReason)
    case notifyKeyRotation(Date?)
    case switchKey
}

/// GotaTun packet tunnel actor.
///
/// Entirely separate from `PacketTunnelActor`. Shares only
/// `PacketTunnelActorProtocol` as the integration surface with
/// `PacketTunnelProvider`.
///
/// Delegates tunnel connection management to a Rust-backed adapter
/// via `GotaTunAdapterProtocol`. Each connection attempt creates a
/// new adapter instance.
public final class GotaTunActor: PacketTunnelActorProtocol, @unchecked Sendable {
    private let logger = Logger(label: "GotaTunActor")

    // MARK: - Dependencies

    private let timings: GotaTunActorTimings
    private let tunnelFd: @Sendable () -> Int32?
    private let applyNetworkSettings: @Sendable (TunnelInterfaceSettings) async throws -> Void
    private let settingsReader: SettingsReaderProtocol
    private let relaySelector: RelaySelectorProtocol
    private let defaultPathObserver: DefaultPathObserverProtocol
    private let blockedStateErrorMapper: BlockedStateErrorMapperProtocol
    private let adapterFactory: GotaTunAdapterFactory
    private var lastAppliedSettings: TunnelInterfaceSettings?

    // MARK: - Internal state (access serialized via eventChannel)

    private var state: GotaTunState = .initial {
        didSet {
            let observed = state.observedState
            _observedState = observed
            for continuation in stateContinuations {
                continuation.yield(observed)
            }
        }
    }

    private var _observedState: ObservedState = .initial
    private var stateContinuations: [AsyncStream<ObservedState>.Continuation] = []
    private var currentAdapter: GotaTunAdapterProtocol?
    private var recoveryTask: AutoCancellingTask?
    private var keySwitchTask: AutoCancellingTask?
    private var priorKey: WireGuard.PrivateKey?
    private var eventLoopTask: Task<Void, Never>?

    // MARK: - Event channel

    private let eventStream: AsyncStream<GotaTunEvent>
    private let eventContinuation: AsyncStream<GotaTunEvent>.Continuation

    // MARK: - Callback proxy

    /// Forwards adapter callbacks into the event channel.
    /// Sendable and safe to call from any thread.
    private final class CallbackProxy: GotaTunCallbackHandler, @unchecked Sendable {
        private let continuation: AsyncStream<GotaTunEvent>.Continuation

        init(continuation: AsyncStream<GotaTunEvent>.Continuation) {
            self.continuation = continuation
        }

        func onConnected() {
            continuation.yield(.adapterConnected)
        }

        func onTimeout() {
            continuation.yield(.adapterTimeout)
        }

        func onError(_ error: GotaTunError) {
            continuation.yield(.adapterError(error))
        }
    }

    private lazy var callbackProxy = CallbackProxy(continuation: eventContinuation)

    // MARK: - Init

    public init(
        timings: GotaTunActorTimings = GotaTunActorTimings(),
        tunnelFd: @Sendable @escaping () -> Int32? = { nil },
        applyNetworkSettings: @Sendable @escaping (TunnelInterfaceSettings) async throws -> Void = { _ in },
        settingsReader: SettingsReaderProtocol,
        relaySelector: RelaySelectorProtocol,
        defaultPathObserver: DefaultPathObserverProtocol,
        blockedStateErrorMapper: BlockedStateErrorMapperProtocol,
        adapterFactory: GotaTunAdapterFactory
    ) {
        self.timings = timings
        self.tunnelFd = tunnelFd
        self.applyNetworkSettings = applyNetworkSettings
        self.settingsReader = settingsReader
        self.relaySelector = relaySelector
        self.defaultPathObserver = defaultPathObserver
        self.blockedStateErrorMapper = blockedStateErrorMapper
        self.adapterFactory = adapterFactory

        (eventStream, eventContinuation) = AsyncStream<GotaTunEvent>.makeStream()

        eventLoopTask = Task { [weak self] in
            guard let self else { return }
            for await event in self.eventStream {
                await self.handleEvent(event)
            }
        }
    }

    deinit {
        eventLoopTask?.cancel()
        eventContinuation.finish()
        for c in stateContinuations { c.finish() }
    }

    // MARK: - PacketTunnelActorProtocol

    public var observedState: ObservedState {
        get async { _observedState }
    }

    public var observedStates: AsyncStream<ObservedState> {
        get async {
            let (stream, continuation) = AsyncStream<ObservedState>.makeStream()
            continuation.yield(_observedState)
            stateContinuations.append(continuation)
            return stream
        }
    }

    public func start(options: StartOptions) {
        eventContinuation.yield(.start(options))
    }

    public func stop() {
        eventContinuation.yield(.stop)
    }

    public func waitUntilDisconnected() async {
        if case .disconnected = state { return }
        let states = await observedStates
        for await s in states {
            if case .disconnected = s { return }
        }
    }

    public func onSleep() {
        currentAdapter?.suspendTunnel()
    }

    public func onWake() {
        currentAdapter?.wakeTunnel()
    }

    public func updateNetworkReachability(networkPathStatus: NWPath.Status) {
        eventContinuation.yield(.networkReachability(networkPathStatus))
    }

    public func reconnect(to nextRelays: NextRelays, reconnectReason: ActorReconnectReason) {
        eventContinuation.yield(.reconnect(nextRelays, reconnectReason))
    }

    public func notifyKeyRotation(date: Date?) {
        eventContinuation.yield(.notifyKeyRotation(date))
    }

    public func setErrorState(reason: BlockedStateReason) {
        eventContinuation.yield(.setErrorState(reason))
    }

    // No-ops — PQ is handled entirely in Rust
    public func notifyEphemeralPeerNegotiated() {}
    public func changeEphemeralPeerNegotiationState(
        configuration: EphemeralPeerNegotiationState,
        reconfigurationSemaphore: OneshotChannel
    ) {}

    // MARK: - Event loop

    private func handleEvent(_ event: GotaTunEvent) async {
        switch event {
        case let .start(options):
            await handleStart(options)
        case .stop:
            handleStop()
        case .adapterConnected:
            handleAdapterConnected()
        case .adapterTimeout:
            await handleAdapterTimeout()
        case let .adapterError(error):
            handleAdapterError(error)
        case let .reconnect(nextRelays, reason):
            await handleReconnect(nextRelays: nextRelays, reason: reason)
        case let .networkReachability(pathStatus):
            handleNetworkReachability(pathStatus)
        case let .setErrorState(reason):
            handleSetErrorState(reason)
        case let .notifyKeyRotation(date):
            handleNotifyKeyRotation(date)
        case .switchKey:
            await handleSwitchKey()
        }
    }

    // MARK: - Start

    private func handleStart(_ options: StartOptions) async {
        guard case .initial = state else {
            logger.debug("Ignoring start() — not in initial state")
            return
        }

        await startConnection(nextRelays: options.selectedRelays.map { .preSelected($0) } ?? .random)
    }

    // MARK: - Stop

    private func handleStop() {
        switch state {
        case .disconnected:
            return
        default:
            stopCurrentAdapter()
            recoveryTask = nil
            keySwitchTask = nil
            priorKey = nil
            state = .disconnected
            logger.debug("Stopped — entering disconnected state")
        }
    }

    // MARK: - Adapter callbacks

    private func handleAdapterConnected() {
        switch state {
        case var .connecting(info):
            info.connectionAttemptCount = 0
            state = .connected(info)
            logger.debug("Connected")
        case var .reconnecting(info):
            info.connectionAttemptCount = 0
            state = .connected(info)
            logger.debug("Reconnected")
        default:
            logger.debug("Ignoring onConnected — state is \(state)")
        }
    }

    private func handleAdapterTimeout() async {
        switch state {
        case var .connecting(info):
            info.incrementAttemptCount()
            logger.debug("Timeout during connecting (attempt \(info.connectionAttemptCount))")
            stopCurrentAdapter()
            state = .connecting(info)
            await startConnection(nextRelays: .random)
        case var .connected(info), var .reconnecting(info):
            info.incrementAttemptCount()
            let wasConnected = {
                if case .connected = self.state { return true }
                return false
            }()
            logger.debug(
                "Timeout — entering \(wasConnected ? "reconnecting" : "reconnecting") (attempt \(info.connectionAttemptCount))"
            )
            stopCurrentAdapter()
            state = .reconnecting(info)
            await startConnection(nextRelays: .random)
        default:
            logger.debug("Ignoring onTimeout — state is \(state)")
        }
    }

    private func handleAdapterError(_ error: GotaTunError) {
        let blockedReason = mapGotaTunError(error)
        logger.error("Adapter error: \(error) → blocked reason: \(blockedReason)")
        enterErrorState(reason: blockedReason)
    }

    // MARK: - Reconnect

    private func handleReconnect(nextRelays: NextRelays, reason: ActorReconnectReason) async {
        switch state {
        case .initial, .disconnected:
            logger.debug("Ignoring reconnect — not connected or connecting")
            return
        case .error(let blocked):
            if blocked.reason == .offline, reason == .userInitiated {
                logger.debug("Ignoring user reconnect — offline")
                return
            }
            logger.debug("Reconnecting from error state")
            recoveryTask = nil
            stopCurrentAdapter()
            await startConnection(nextRelays: nextRelays)
        case .connecting, .connected, .reconnecting:
            logger.debug("Reconnecting (reason: \(reason))")
            stopCurrentAdapter()
            if case .connectionLoss = reason {
                // Handled via adapter timeout, preserve state
                return
            }
            await startConnection(nextRelays: nextRelays)
        }
    }

    // MARK: - Network reachability

    private func handleNetworkReachability(_ pathStatus: NWPath.Status) {
        let newReachability: NetworkReachability = pathStatus == .satisfied ? .reachable : .unreachable

        switch state {
        case var .connecting(info):
            info.networkReachability = newReachability
            state = .connecting(info)
            if newReachability == .unreachable {
                enterErrorState(reason: .offline)
            } else {
                currentAdapter?.recycleUdpSockets()
            }
        case var .connected(info):
            info.networkReachability = newReachability
            state = .connected(info)
            if newReachability == .unreachable {
                enterErrorState(reason: .offline)
            } else {
                currentAdapter?.recycleUdpSockets()
            }
        case var .reconnecting(info):
            info.networkReachability = newReachability
            state = .reconnecting(info)
            if newReachability == .unreachable {
                enterErrorState(reason: .offline)
            } else {
                currentAdapter?.recycleUdpSockets()
            }
        case var .error(blocked):
            blocked.networkReachability = newReachability
            state = .error(blocked)
            if newReachability == .reachable, blocked.reason == .offline {
                logger.debug("Network reachable — leaving offline error state")
                recoveryTask = nil
                // Route through event channel to stay serialized
                eventContinuation.yield(.reconnect(.random, .restoredConnectivity))
            }
        default:
            break
        }
    }

    // MARK: - Error state

    private func handleSetErrorState(_ reason: BlockedStateReason) {
        enterErrorState(reason: reason)
    }

    private func enterErrorState(reason: BlockedStateReason) {
        if case .disconnected = state { return }

        let priorState: GotaTunBlockedInfo.PriorState
        let relayConstraints: RelayConstraints?
        let reachability: NetworkReachability

        switch state {
        case .initial:
            priorState = .initial
            relayConstraints = nil
            reachability = .undetermined
        case let .connecting(info):
            priorState = .connecting
            relayConstraints = info.relayConstraints
            reachability = info.networkReachability
        case let .connected(info):
            priorState = .connected
            relayConstraints = info.relayConstraints
            reachability = info.networkReachability
        case let .reconnecting(info):
            priorState = .reconnecting
            relayConstraints = info.relayConstraints
            reachability = info.networkReachability
        case let .error(blocked):
            var updated = blocked
            updated.reason = reason
            state = .error(updated)
            return
        case .disconnected:
            return
        }

        stopCurrentAdapter()
        keySwitchTask = nil

        let blocked = GotaTunBlockedInfo(
            reason: reason,
            relayConstraints: relayConstraints,
            priorState: priorState,
            networkReachability: reachability
        )
        state = .error(blocked)
        logger.debug("Entering error state: \(reason)")

        if reason.recoverableError() {
            startRecoveryTask()
        }
    }

    // MARK: - Key rotation

    private func handleNotifyKeyRotation(_ date: Date?) {
        guard case .connected = state else {
            // If not connected, just reconnect with new key on next attempt
            return
        }

        // Cache the current key as "prior" and schedule switch after propagation delay
        do {
            let settings = try settingsReader.read()
            priorKey = settings.privateKey
        } catch {
            logger.error("Failed to read settings for key rotation: \(error)")
            return
        }

        logger.debug("Key rotation notified — caching prior key, scheduling switch in \(timings.wgKeyPropagationDelay)")

        keySwitchTask = AutoCancellingTask(
            Task { [weak self, timings] in
                try? await Task.sleep(for: timings.wgKeyPropagationDelay)
                guard !Task.isCancelled else { return }
                self?.eventContinuation.yield(.switchKey)
            })
    }

    private func handleSwitchKey() async {
        priorKey = nil
        keySwitchTask = nil

        switch state {
        case .connected, .connecting, .reconnecting:
            logger.debug("Key propagation delay elapsed — reconnecting with new key")
            stopCurrentAdapter()
            await startConnection(nextRelays: .random)
        default:
            break
        }
    }

    // MARK: - Connection management

    private func startConnection(nextRelays: NextRelays) async {
        let settings: Settings
        do {
            settings = try settingsReader.read()
        } catch {
            let reason = blockedStateErrorMapper.mapError(error)
            enterErrorState(reason: reason)
            return
        }

        let selectedRelays: SelectedRelays
        do {
            let attemptCount: UInt
            switch state {
            case let .connecting(info), let .reconnecting(info):
                attemptCount = info.connectionAttemptCount
            default:
                attemptCount = 0
            }

            switch nextRelays {
            case .random:
                selectedRelays = try relaySelector.selectRelays(
                    tunnelSettings: settings.tunnelSettings,
                    connectionAttemptCount: attemptCount
                )
            case .current:
                if let currentRelays = currentConnectionInfo?.selectedRelays {
                    selectedRelays = currentRelays
                } else {
                    selectedRelays = try relaySelector.selectRelays(
                        tunnelSettings: settings.tunnelSettings,
                        connectionAttemptCount: attemptCount
                    )
                }
            case let .preSelected(relays):
                selectedRelays = relays
            }
        } catch {
            let reason = blockedStateErrorMapper.mapError(error)
            enterErrorState(reason: reason)
            return
        }

        let privateKey = priorKey ?? settings.privateKey

        // Apply network settings (DNS, addresses) to iOS before starting the adapter
        let interfaceSettings = settings.interfaceSettings()
        if interfaceSettings != lastAppliedSettings {
            do {
                try await applyNetworkSettings(interfaceSettings)
                lastAppliedSettings = interfaceSettings
                logger.debug("Applied tunnel network settings")
            } catch {
                logger.error("Failed to apply network settings: \(error)")
                enterErrorState(reason: .tunnelAdapter)
                return
            }
        }

        guard let fd = tunnelFd() else {
            logger.error("Failed to obtain tunnel file descriptor")
            enterErrorState(reason: .tunnelAdapter)
            return
        }

        // Compute establish timeout with exponential backoff
        let establishTimeout = Self.computeEstablishTimeout(attemptCount: currentAttemptCount)

        // Obfuscation is handled in Rust — pass obfuscation method through config.
        // The Rust adapter will start the obfuscation proxy internally.
        let config = GotaTunConfig(
            tunnelFd: fd,
            privateKey: privateKey.rawValue,
            interfaceAddresses: settings.interfaceAddresses.map { $0.description },
            mtu: 1280,
            ipv4Gateway: "\(selectedRelays.exit.endpoint.ipv4Gateway)",
            clientPublicKey: settings.privateKey.publicKey.rawValue,
            exitPeerPublicKey: selectedRelays.exit.endpoint.publicKey,
            exitPeerEndpoint: "\(selectedRelays.exit.endpoint.socketAddress)",
            entryPeerPublicKey: selectedRelays.entry?.endpoint.publicKey,
            entryPeerEndpoint: selectedRelays.entry.map { "\($0.endpoint.socketAddress)" },
            isPostQuantum: settings.tunnelSettings.tunnelQuantumResistance.isEnabled,
            isDaitaEnabled: settings.tunnelSettings.daita.isEnabled,
            establishTimeout: establishTimeout,
            obfuscationMethod: selectedRelays.ingress.endpoint.obfuscation
        )

        let connectionInfo = GotaTunConnectionInfo(
            selectedRelays: selectedRelays,
            relayConstraints: settings.tunnelSettings.relayConstraints,
            networkReachability: currentReachability,
            connectionAttemptCount: currentAttemptCount,
            transportLayer: .udp,
            remotePort: selectedRelays.exit.endpoint.socketAddress.port,
            lastKeyRotation: currentLastKeyRotation,
            isPostQuantum: config.isPostQuantum,
            isDaitaEnabled: config.isDaitaEnabled
        )

        // Determine target state
        let wasConnected: Bool
        switch state {
        case .connected, .reconnecting:
            wasConnected = true
        default:
            wasConnected = false
        }

        if wasConnected {
            state = .reconnecting(connectionInfo)
        } else {
            state = .connecting(connectionInfo)
        }

        // Create and start adapter
        let adapter = adapterFactory.makeAdapter()
        currentAdapter = adapter

        do {
            try adapter.startTunnel(config: config, callbackHandler: callbackProxy)
        } catch {
            logger.error("Failed to start tunnel: \(error)")
            currentAdapter = nil
            enterErrorState(reason: .tunnelAdapter)
        }
    }

    private func stopCurrentAdapter() {
        currentAdapter?.stopTunnel()
        currentAdapter = nil
    }

    // MARK: - Recovery

    private func startRecoveryTask() {
        recoveryTask = AutoCancellingTask(
            Task { [weak self, timings] in
                while !Task.isCancelled {
                    try? await Task.sleep(for: timings.bootRecoveryPeriodicity)
                    guard !Task.isCancelled else { return }
                    self?.eventContinuation.yield(.reconnect(.random, .userInitiated))
                }
            })
    }

    // MARK: - Helpers

    private var currentConnectionInfo: GotaTunConnectionInfo? {
        switch state {
        case let .connecting(info), let .connected(info), let .reconnecting(info):
            return info
        default:
            return nil
        }
    }

    private var currentReachability: NetworkReachability {
        switch state {
        case let .connecting(info), let .connected(info), let .reconnecting(info):
            return info.networkReachability
        case let .error(blocked):
            return blocked.networkReachability
        default:
            return .undetermined
        }
    }

    private var currentAttemptCount: UInt {
        currentConnectionInfo?.connectionAttemptCount ?? 0
    }

    private var currentLastKeyRotation: Date? {
        currentConnectionInfo?.lastKeyRotation
    }

    private static func computeEstablishTimeout(attemptCount: UInt) -> UInt32 {
        let base: UInt32 = 4
        let multiplier = UInt32(1) << UInt32(min(attemptCount, 3))  // 1, 2, 4, 8
        let timeout = base.saturating_multiplying(by: multiplier)
        return min(timeout, 15)  // Cap at 15 seconds
    }

    private func mapGotaTunError(_ error: GotaTunError) -> BlockedStateReason {
        switch error {
        case .ephemeralPeerNegotiation, .obfuscationSetup, .invalidConfig:
            return .tunnelAdapter
        case .internalError:
            return .unknown
        }
    }
}

private extension UInt32 {
    func saturating_multiplying(by other: UInt32) -> UInt32 {
        let (result, overflow) = self.multipliedReportingOverflow(by: other)
        return overflow ? .max : result
    }
}
