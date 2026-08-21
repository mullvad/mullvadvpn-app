//
//  GotaTunActor.swift
//  PacketTunnelCore
//
//  Created by Emīls on 2026-08-13.
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
    case adapterConnected(generation: UInt64)
    case adapterTimeout(generation: UInt64)
    case adapterError(GotaTunError, generation: UInt64)
    case reconnect(NextRelays, ActorReconnectReason)
    case networkReachability(NWPath.Status)
    case setErrorState(BlockedStateReason)
    case sleep
    case wake
    #if NEVER_IN_PRODUCTION
        /// Resolved once every event queued ahead of it has been handled. Lets tests observe the
        /// actor at a known point instead of waiting for a duration.
        case barrier(@Sendable () -> Void)
    #endif
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
public actor GotaTunActor: PacketTunnelActorProtocol {
    private let logger = Logger(label: "GotaTunActor")

    // MARK: - Dependencies

    private let providerDelegate: TunnelProviderDelegate
    private let settingsReader: SettingsReaderProtocol
    private let relaySelector: RelaySelectorProtocol
    private let defaultPathObserver: GotaTunPathObserverProtocol
    private let blockedStateErrorMapper: BlockedStateErrorMapperProtocol
    private let adapterFactory: GotaTunAdapterFactory
    private var lastAppliedSettings: TunnelInterfaceSettings?

    // MARK: - State (mutated only from the event loop)

    public private(set) var observedState: ObservedState = .initial {
        didSet {
            guard observedState != oldValue else { return }
            stateBroadcaster.send(observedState)
            if case .disconnected = observedState {
                terminate()
            }
        }
    }

    private var stateBroadcaster = ObservedStateBroadcaster()
    /// Unknown until the path monitor delivers its first update, which is what releases `start`.
    private var currentReachability: NetworkReachability = .undetermined
    private var currentAdapter: GotaTunAdapterProtocol?
    /// Incremented whenever the current adapter is replaced or stopped. Events tagged with an
    /// older generation come from an adapter no longer in use and are dropped.
    private var adapterGeneration: UInt64 = 0

    // MARK: - Event channel

    private let eventStream: AsyncStream<GotaTunEvent>
    private let eventContinuation: AsyncStream<GotaTunEvent>.Continuation

    // MARK: - Callback proxy

    /// Forwards adapter callbacks into the event channel, tagged with the generation of the
    /// adapter they came from. Sendable and safe to call from any thread.
    private final class CallbackProxy: GotaTunCallbackHandler {
        private let continuation: AsyncStream<GotaTunEvent>.Continuation
        private let generation: UInt64

        init(continuation: AsyncStream<GotaTunEvent>.Continuation, generation: UInt64) {
            self.continuation = continuation
            self.generation = generation
        }

        func onConnected() {
            continuation.yield(.adapterConnected(generation: generation))
        }

        func onTimeout() {
            continuation.yield(.adapterTimeout(generation: generation))
        }

        func onError(_ error: GotaTunError) {
            continuation.yield(.adapterError(error, generation: generation))
        }
    }

    // MARK: - Init

    public init(
        providerDelegate: sending TunnelProviderDelegate,
        settingsReader: SettingsReaderProtocol,
        relaySelector: RelaySelectorProtocol,
        defaultPathObserver: GotaTunPathObserverProtocol = GotaTunPathObserver(),
        blockedStateErrorMapper: BlockedStateErrorMapperProtocol,
        adapterFactory: GotaTunAdapterFactory
    ) {
        self.providerDelegate = providerDelegate
        self.settingsReader = settingsReader
        self.relaySelector = relaySelector
        self.defaultPathObserver = defaultPathObserver
        self.blockedStateErrorMapper = blockedStateErrorMapper
        self.adapterFactory = adapterFactory

        (eventStream, eventContinuation) = AsyncStream<GotaTunEvent>.makeStream()

        consumeEvents()
    }

    deinit {
        Task { [defaultPathObserver] in await defaultPathObserver.stop() }
        eventContinuation.finish()
    }

    /// Starts path observation and consumes actor events.
    /// The events are consumed in a detached task until the event stream is finished. This function **must** be called excatly once per the lifetime of an actor. Without this, the actor will never act on events, so no work will actually be done.
    /// `self` is resolved weakly per iteration so the loop does not keep the actor alive.
    private nonisolated consuming func consumeEvents() {
        Task.detached { [weak self, eventStream] in
            await self?.startPathObservation()
            for await event in eventStream {
                guard let self else { return }
                await self.handleEvent(event)
            }
        }
    }

    private func startPathObservation() async {
        let eventContinuation = self.eventContinuation
        currentReachability = await defaultPathObserver.start { status in
            eventContinuation.yield(.networkReachability(status))
        }.networkReachability
    }

    // MARK: - PacketTunnelActorProtocol

    public var observedStates: AsyncStream<ObservedState> {
        stateBroadcaster.makeStream(replaying: observedState)
    }

    nonisolated public func start(options: StartOptions) async {
        eventContinuation.yield(.start(options))
    }

    nonisolated public func stop() {
        eventContinuation.yield(.stop)
    }

    public func waitUntilDisconnected() async {
        await observedStates.waitUntilDisconnected()
    }

    nonisolated public func onSleep() {
        eventContinuation.yield(.sleep)
    }

    nonisolated public func onWake() {
        eventContinuation.yield(.wake)
    }

    /// Ignored: reachability handling comes with the actor's own path observation.
    nonisolated public func updateNetworkReachability(networkPathStatus: NWPath.Status) {}

    #if NEVER_IN_PRODUCTION
        /// Returns once every event queued before this call has been handled.
        /// Returns immediately once the actor has stopped, since no further events can run.
        nonisolated func drainEvents() async {
            await withCheckedContinuation { continuation in
                // A finished stream silently drops the barrier, so resume rather than wait forever.
                guard case .enqueued = eventContinuation.yield(.barrier { continuation.resume() })
                else {
                    continuation.resume()
                    return
                }
            }
        }
    #endif

    nonisolated public func reconnect(to nextRelays: NextRelays, reconnectReason: ActorReconnectReason) {
        eventContinuation.yield(.reconnect(nextRelays, reconnectReason))
    }

    // TODO: implement key rotation handling
    nonisolated public func notifyKeyRotation(date: Date?) {}

    nonisolated public func setErrorState(reason: BlockedStateReason) {
        eventContinuation.yield(.setErrorState(reason))
    }

    // No-ops: PQ is handled entirely in Rust. Remove when the WireGuardGo interface is removed.
    nonisolated public func notifyEphemeralPeerNegotiated() {}
    nonisolated public func changeEphemeralPeerNegotiationState(
        configuration: EphemeralPeerNegotiationState,
        reconfigurationSemaphore: OneshotChannel
    ) {}

    // MARK: - Event loop

    private func handleEvent(_ event: GotaTunEvent) async {
        switch event {
        case let .start(options):
            await handleStart(options)
        case .stop:
            await handleStop()
        case let .adapterConnected(generation):
            guard isCurrentAdapter(generation) else { return }
            await handleAdapterConnected()
        case let .adapterTimeout(generation):
            guard isCurrentAdapter(generation) else { return }
            await handleAdapterTimeout()
        case let .adapterError(error, generation):
            guard isCurrentAdapter(generation) else { return }
            handleAdapterError(error)
        case let .reconnect(nextRelays, reason):
            await handleReconnect(nextRelays: nextRelays, reason: reason)
        case let .networkReachability(pathStatus):
            await handleNetworkReachability(pathStatus)
        case let .setErrorState(reason):
            handleSetErrorState(reason)
        case .sleep:
            currentAdapter?.suspendTunnel()
        case .wake:
            currentAdapter?.wakeTunnel()
        #if NEVER_IN_PRODUCTION
            case let .barrier(resume):
                resume()
        #endif
        }
    }

    // MARK: - Start

    private func handleStart(_ options: StartOptions) async {
        guard case .initial = observedState else {
            logger.debug("Ignoring start() while not in initial state")
            return
        }

        guard !isDeviceOffline else {
            logger.debug("Starting while offline, entering blocked state")
            enterErrorState(reason: .offline)
            return
        }

        await startConnection(nextRelays: options.selectedRelays.map { .preSelected($0) } ?? .random)
    }

    // MARK: - Stop

    private func handleStop() async {
        switch observedState {
        case .disconnected:
            return
        default:
            stopCurrentAdapter()
            await defaultPathObserver.stop()
            observedState = .disconnected
            logger.debug("Stopped, entering disconnected state")
        }
    }

    /// Finish the event loop. Called upon entering `.disconnected`.
    /// Observation streams are finished by the broadcaster.
    private func terminate() {
        eventContinuation.finish()
    }

    // MARK: - Adapter callbacks

    private func handleAdapterConnected() async {
        switch observedState {
        case var .connecting(info), var .reconnecting(info):
            info.connectionAttemptCount = 0
            observedState = .connected(info)
            await providerDelegate.reassertTunnel()
            logger.debug("Connected")
        default:
            logger.debug("Ignoring onConnected in state \(observedState)")
        }
    }

    private func handleAdapterTimeout() async {
        guard observedState.connectionState != nil else {
            logger.debug("Ignoring onTimeout in state \(observedState)")
            return
        }

        await restartConnection(nextRelays: .random, incrementAttempt: true)
    }

    private func handleAdapterError(_ error: GotaTunError) {
        let blockedReason = mapGotaTunError(error)
        logger.error("Adapter error: \(error) → blocked reason: \(blockedReason)")
        enterErrorState(reason: blockedReason)
    }

    // MARK: - Reconnect

    private func handleReconnect(nextRelays: NextRelays, reason: ActorReconnectReason) async {
        switch observedState {
        case let .error(blocked):
            if blocked.reason == .offline, isDeviceOffline {
                logger.debug("Ignoring reconnect while offline")
                return
            }
            logger.debug("Reconnecting from error state")
            stopCurrentAdapter()
            await startConnection(nextRelays: nextRelays)
        case .connecting, .connected, .reconnecting:
            logger.debug("Reconnecting (reason: \(reason))")
            await restartConnection(nextRelays: nextRelays)
        default:
            logger.debug("Ignoring reconnect, not connected or connecting")
        }
    }

    // MARK: - Network reachability

    private func handleNetworkReachability(_ pathStatus: NWPath.Status) async {
        let newReachability = pathStatus.networkReachability

        guard newReachability != .undetermined else {
            logger.debug("Ignoring unknown network path status: \(pathStatus)")
            return
        }

        let previousReachability = currentReachability
        currentReachability = newReachability

        switch observedState {
        case .connecting, .connected, .reconnecting:
            guard newReachability != .unreachable else {
                logger.debug("Network unreachable, entering blocked state")
                enterErrorState(reason: .offline)
                return
            }

            observedState.mutateConnectionState { $0.networkReachability = newReachability }
            currentAdapter?.recycleUdpSockets()

        case let .error(blocked):
            // Restoration requires connectivity to have been lost first, otherwise the first
            // update after starting would masquerade as a recovery and pre-empt the retry timer.
            guard newReachability == .reachable, previousReachability == .unreachable,
                blocked.reason.recoverableError()
            else { return }
            logger.debug("Network reachable, restoring connectivity from \(blocked.reason)")
            await startConnection(nextRelays: .random)

        default:
            return
        }
    }

    // MARK: - Error state

    private func handleSetErrorState(_ reason: BlockedStateReason) {
        enterErrorState(reason: reason)
    }

    private func enterErrorState(reason: BlockedStateReason) {
        switch observedState {
        case .disconnected:
            return

        case let .error(blocked) where blocked.reason == reason:
            break

        case var .error(blocked):
            blocked.reason = reason
            observedState = .error(blocked)
            logger.debug("Updating error state reason: \(reason)")

        default:
            stopCurrentAdapter()
            observedState = .error(
                ObservedBlockedState(
                    reason: reason,
                    relayConstraints: observedState.connectionState?.relayConstraints
                ))
            logger.debug("Entering error state: \(reason)")
        }
    }

    // MARK: - Connection management

    /// Replace the current connection attempt with a fresh one, carrying over the attempt count
    /// and whether the tunnel was already established. No-op unless a connection is in progress.
    private func restartConnection(nextRelays: NextRelays, incrementAttempt: Bool = false) async {
        guard var info = observedState.connectionState else { return }

        if incrementAttempt {
            info.incrementAttemptCount()
        }
        let isReconnect = isTunnelEstablished
        logger.debug("Restarting connection (attempt \(info.connectionAttemptCount))")

        stopCurrentAdapter()
        await startConnection(
            nextRelays: nextRelays,
            attemptCount: info.connectionAttemptCount,
            isReconnect: isReconnect
        )
    }

    private func startConnection(
        nextRelays: NextRelays,
        attemptCount: UInt = 0,
        isReconnect: Bool = false
    ) async {
        let settings: Settings
        do {
            settings = try settingsReader.read()
        } catch {
            enterErrorState(reason: blockedStateErrorMapper.mapError(error))
            return
        }

        let selectedRelays: SelectedRelays
        do {
            selectedRelays = try selectRelays(nextRelays, settings: settings, attemptCount: attemptCount)
        } catch {
            enterErrorState(reason: blockedStateErrorMapper.mapError(error))
            return
        }

        do {
            try await applyInterfaceSettingsIfNeeded(settings)
        } catch {
            logger.error("Failed to apply network settings: \(error)")
            enterErrorState(reason: .tunnelAdapter)
            return
        }

        guard let fd = providerDelegate.tunnelFileDescriptor else {
            logger.error("Failed to obtain tunnel file descriptor")
            enterErrorState(reason: .tunnelAdapter)
            return
        }

        guard let (ipv4Address, ipv6Address) = interfaceAddresses(in: settings) else {
            logger.error("Failed to extract both IPv4 and IPv6 address from settings")
            enterErrorState(reason: .tunnelAdapter)
            return
        }

        let config = Self.makeGotaTunConfig(
            settings: settings,
            selectedRelays: selectedRelays,
            privateKey: settings.privateKey,
            fd: fd,
            ipv4Address: ipv4Address,
            ipv6Address: ipv6Address,
            establishTimeout: Self.computeEstablishTimeout(attemptCount: attemptCount)
        )

        let connectionState = Self.makeConnectionState(
            config: config,
            selectedRelays: selectedRelays,
            relayConstraints: settings.tunnelSettings.relayConstraints,
            networkReachability: currentReachability,
            connectionAttemptCount: attemptCount,
            lastKeyRotation: nil
        )

        observedState = isReconnect ? .reconnecting(connectionState) : .connecting(connectionState)

        startAdapter(config: config)
    }

    private func selectRelays(
        _ nextRelays: NextRelays,
        settings: Settings,
        attemptCount: UInt
    ) throws -> SelectedRelays {
        if case let .preSelected(relays) = nextRelays {
            return relays
        }
        if case .current = nextRelays {
            logger.warning("NextRelays.current is unsupported by GotaTun, selecting new relays")
        }
        return try relaySelector.selectRelays(
            tunnelSettings: settings.tunnelSettings,
            connectionAttemptCount: attemptCount
        )
    }

    /// Pushes `settings`' interface settings (DNS, addresses) to iOS, skipping the call if unchanged.
    private func applyInterfaceSettingsIfNeeded(_ settings: Settings) async throws {
        let interfaceSettings = settings.interfaceSettings()
        guard interfaceSettings != lastAppliedSettings else { return }
        try await providerDelegate.applyNetworkSettings(interfaceSettings)
        lastAppliedSettings = interfaceSettings
        logger.debug("Applied tunnel network settings")
    }

    private func startAdapter(config: GotaTunConfig) {
        let adapter = adapterFactory.makeAdapter()
        adapterGeneration += 1
        currentAdapter = adapter

        do {
            try adapter.startTunnel(
                config: config,
                callbackHandler: CallbackProxy(continuation: eventContinuation, generation: adapterGeneration)
            )
        } catch {
            logger.error("Failed to start tunnel: \(error)")
            currentAdapter = nil
            enterErrorState(reason: .tunnelAdapter)
        }
    }

    private func stopCurrentAdapter() {
        currentAdapter?.stopTunnel()
        currentAdapter = nil
        adapterGeneration += 1
    }

    /// Whether `generation` identifies the adapter currently in use. Events from a replaced or
    /// stopped adapter can still be queued in the event stream; they must not be applied to the
    /// current attempt.
    private func isCurrentAdapter(_ generation: UInt64) -> Bool {
        guard generation == adapterGeneration else {
            logger.debug("Ignoring event from stale adapter")
            return false
        }
        return true
    }

    // MARK: - Pure config building

    private func interfaceAddresses(in settings: Settings) -> (IPv4Address, IPv6Address)? {
        guard let ipv4 = settings.interfaceAddresses.lazy.compactMap({ IPv4Address("\($0.address)") }).first
        else {
            logger.error("No IPv4 interface address available")
            return nil
        }

        guard let ipv6 = settings.interfaceAddresses.lazy.compactMap({ IPv6Address("\($0.address)") }).first
        else {
            logger.error("No IPv6 interface address available")
            return nil
        }

        return (ipv4, ipv6)
    }

    private static func makeGotaTunConfig(
        settings: Settings,
        selectedRelays: SelectedRelays,
        privateKey: WireGuard.PrivateKey,
        fd: Int32,
        ipv4Address: IPv4Address,
        ipv6Address: IPv6Address,
        establishTimeout: UInt32
    ) -> GotaTunConfig {
        GotaTunConfig(
            tunnelFd: fd,
            privateKey: privateKey.rawValue,
            ipv4Address: ipv4Address,
            ipv6Address: ipv6Address,
            mtu: 1280,
            ipv4Gateway: "\(selectedRelays.exit.endpoint.ipv4Gateway)",
            clientPublicKey: privateKey.publicKey.rawValue,
            exitPeerPublicKey: selectedRelays.exit.endpoint.publicKey,
            exitPeerEndpoint: "\(selectedRelays.exit.endpoint.socketAddress)",
            entryPeerPublicKey: selectedRelays.entry?.endpoint.publicKey,
            entryPeerEndpoint: selectedRelays.entry.map { "\($0.endpoint.socketAddress)" },
            isPostQuantum: settings.tunnelSettings.tunnelQuantumResistance.isEnabled,
            isDaitaEnabled: settings.tunnelSettings.daita.isEnabled,
            establishTimeout: establishTimeout,
            obfuscationMethod: selectedRelays.ingress.endpoint.obfuscation
        )
    }

    private static func makeConnectionState(
        config: GotaTunConfig,
        selectedRelays: SelectedRelays,
        relayConstraints: RelayConstraints,
        networkReachability: NetworkReachability,
        connectionAttemptCount: UInt,
        lastKeyRotation: Date?
    ) -> ObservedConnectionState {
        ObservedConnectionState(
            selectedRelays: selectedRelays,
            relayConstraints: relayConstraints,
            networkReachability: networkReachability,
            connectionAttemptCount: connectionAttemptCount,
            transportLayer: config.obfuscationMethod.transportLayer,
            remotePort: selectedRelays.ingress.endpoint.socketAddress.port,
            lastKeyRotation: lastKeyRotation,
            isPostQuantum: config.isPostQuantum,
            isDaitaEnabled: config.isDaitaEnabled,
            obfuscationMethod: selectedRelays.obfuscation,
        )
    }

    // MARK: - Helpers

    private var isDeviceOffline: Bool {
        currentReachability == .unreachable
    }

    /// Whether the tunnel was already up, so a fresh attempt is reported as `.reconnecting`.
    private var isTunnelEstablished: Bool {
        switch observedState {
        case .connected, .reconnecting:
            return true
        default:
            return false
        }
    }

    private static func computeEstablishTimeout(attemptCount: UInt) -> UInt32 {
        let base: UInt32 = 4
        return min(base << min(attemptCount, 2), 15)  // 4, 8, 15 seconds
    }

    private func mapGotaTunError(_ error: GotaTunError) -> BlockedStateReason {
        switch error {
        case .invalidConfig:
            return .tunnelAdapter
        case .internalError:
            return .unknown
        }
    }
}
