//
//  PacketTunnelActor.swift
//  PacketTunnel
//
//  Created by pronebird on 30/06/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
@preconcurrency import MullvadLogging
import MullvadREST
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes
import NetworkExtension
import WireGuardKitTypes

/**
 Packet tunnel state machine implemented as an actor.

 - Actor receives events for execution over the `EventChannel`.

 - Events are consumed in a detached task via for-await loop over the channel. Each event, once received, is executed in its entirety before the next
 event is processed. See the implementation of `consumeEvents()` which is the central task dispatcher inside of actor.

 - Most of calls that actor performs suspend for a very short amount of time. `EventChannel` proactively discards unwanted tasks as they arrive to prevent
 future execution, such as repeating commands to reconnect are coalesced and all events prior to stop are discarded entirely as the outcome would be the
 same anyway.
 */
public actor PacketTunnelActor {
    var state: State = .initial {
        didSet(oldValue) {
            guard state != oldValue else { return }
            logger.debug("\(state.logFormat())")
            observedState = state.observedState
        }
    }

    @Published internal(set) public var observedState: ObservedState = .initial

    nonisolated(unsafe) let logger = Logger(label: "PacketTunnelActor")

    let timings: PacketTunnelActorTimings
    let tunnelAdapter: TunnelAdapterProtocol
    let tunnelMonitor: TunnelMonitorProtocol
    let defaultPathObserver: DefaultPathObserverProtocol
    let blockedStateErrorMapper: BlockedStateErrorMapperProtocol
    public let relaySelector: RelaySelectorProtocol
    let settingsReader: SettingsReaderProtocol
    let protocolObfuscator: ProtocolObfuscation
    var lastAppliedTunnelSettings: TunnelInterfaceSettings?

    nonisolated let eventChannel = EventChannel()

    public init(
        timings: PacketTunnelActorTimings,
        tunnelAdapter: TunnelAdapterProtocol,
        tunnelMonitor: TunnelMonitorProtocol,
        defaultPathObserver: DefaultPathObserverProtocol,
        blockedStateErrorMapper: BlockedStateErrorMapperProtocol,
        relaySelector: RelaySelectorProtocol,
        settingsReader: SettingsReaderProtocol,
        protocolObfuscator: ProtocolObfuscation
    ) {
        self.timings = timings
        self.tunnelAdapter = tunnelAdapter
        self.tunnelMonitor = tunnelMonitor
        self.defaultPathObserver = defaultPathObserver
        self.blockedStateErrorMapper = blockedStateErrorMapper
        self.relaySelector = relaySelector
        self.settingsReader = settingsReader
        self.protocolObfuscator = protocolObfuscator

        consumeEvents(channel: eventChannel)
    }

    deinit {
        eventChannel.finish()
    }

    public func isErrorState() async -> Bool {
        if case .error = self.state {
            return true
        }
        return false
    }

    /**
     Spawn a detached task that consumes events from the channel indefinitely until the channel is closed.
     Events are processed one at a time, so no suspensions should affect the order of execution and thus guarantee transactional execution.

     - Parameter channel: event channel.
     */
    private nonisolated func consumeEvents(channel: EventChannel) {
        Task.detached { [weak self] in
            for await event in channel {
                guard let self else { return }
                await self.handleEvent(event)
            }
        }
    }

    private func handleEvent(_ event: Event) async {
        self.logger.debug("Received event: \(event.logFormat())")

        let effects = self.runReducer(event)

        for effect in effects {
            await executeEffect(effect)
        }
    }

    func executeEffect(_ effect: Effect) async {
        switch effect {
        case .startTunnelMonitor:
            setTunnelMonitorEventHandler()
        case .stopTunnelMonitor:
            tunnelMonitor.stop()
        case let .updateTunnelMonitorPath(networkPath):
            await handleDefaultPathChange(networkPath)
        case let .startConnection(nextRelays):
            await handleStartConnection(nextRelays: nextRelays)
        case let .restartConnection(nextRelays, reason):
            await handleRestartConnection(nextRelays: nextRelays, reason: reason)
        case let .reconnect(nextRelay):
            eventChannel.send(.reconnect(nextRelay))
        case .stopTunnelAdapter:
            await handleStopTunnelAdapter()
        case let .configureForErrorState(reason):
            await setErrorStateInternal(with: reason)
        case let .cacheActiveKey(lastKeyRotation):
            cacheActiveKey(lastKeyRotation: lastKeyRotation)
        case let .reconfigureForEphemeralPeer(configuration, configurationSemaphore):
            await handleReconfigureForEphemeralPeer(configuration: configuration, semaphore: configurationSemaphore)
        case .connectWithEphemeralPeer:
            await connectWithEphemeralPeer()
        case .setDisconnectedState:
            self.state = .disconnected
        }
    }

    private func handleStartConnection(nextRelays: NextRelays) async {
        do {
            try await tryStart(nextRelays: nextRelays)
        } catch {
            logger.error(error: error, message: "Failed to start the tunnel.")
            await setErrorStateInternal(with: error)
        }
    }

    private func handleRestartConnection(nextRelays: NextRelays, reason: ActorReconnectReason) async {
        do {
            try await tryStart(nextRelays: nextRelays, reason: reason)
        } catch {
            logger.error(error: error, message: "Failed to reconnect the tunnel.")
            await setErrorStateInternal(with: error)
        }
    }

    private func handleStopTunnelAdapter() async {
        do {
            try await tunnelAdapter.stop()
        } catch {
            logger.error(error: error, message: "Failed to stop adapter.")
        }
        state = .disconnected
    }

    private func handleReconfigureForEphemeralPeer(
        configuration: EphemeralPeerNegotiationState,
        semaphore: OneshotChannel
    ) async {
        do {
            try await updateEphemeralPeerNegotiationState(configuration: configuration)
        } catch {
            logger.error(
                error: error,
                message: "Failed to reconfigure tunnel after ephemeral peer negotiation. Entering error state.")
            // Log the specific error type for debugging
            await setErrorStateInternal(with: error)
        }
        semaphore.send()
    }

    private func handleDefaultPathChange(_ networkPath: Network.NWPath.Status) async {
        tunnelMonitor.handleNetworkPathUpdate(networkPath)

        let newReachability = networkPath.networkReachability

        let reachabilityChanged =
            state.mutateAssociatedData {
                let reachabilityChanged = $0.networkReachability != newReachability
                $0.networkReachability = newReachability
                return reachabilityChanged
            } ?? false
        if case .reachable = newReachability,
            case let .error(
                errorState
            ) = state,
            errorState.reason
                .recoverableError(), reachabilityChanged
        {
            await handleRestartConnection(nextRelays: .random, reason: .userInitiated)
        }
    }
}

// MARK: -

extension PacketTunnelActor {
    /**
     Start the tunnel.

     Can only be called once, all subsequent attempts are ignored. Use `reconnect()` if you wish to change relay.

     - Parameter options: start options produced by packet tunnel
     */
    private func start(options: StartOptions) async {
        guard case .initial = state else { return }

        logger.debug("\(options.logFormat())")

        // Assign a closure receiving tunnel monitor events.
        setTunnelMonitorEventHandler()

        do {
            try await tryStart(nextRelays: options.selectedRelays.map { .preSelected($0) } ?? .random)
        } catch {
            logger.error(error: error, message: "Failed to start the tunnel.")

            await setErrorStateInternal(with: error)
        }
    }

    /// Stop the tunnel.
    private func stop() async {
        switch state {
        case let .connected(connState), let .connecting(connState), let .reconnecting(connState),
            let .negotiatingEphemeralPeer(connState, _):
            state = .disconnecting(connState)
            tunnelMonitor.stop()

            // Fallthrough to stop adapter and shift to `.disconnected` state.
            fallthrough

        case .error:
            do {
                try await tunnelAdapter.stop()
            } catch {
                logger.error(error: error, message: "Failed to stop adapter.")
            }
            state = .disconnected

        case .initial, .disconnected:
            break

        case .disconnecting:
            assertionFailure("stop(): out of order execution.")
        }
    }

    /**
     Entry point for attempting to start the tunnel by performing the following steps:

     - Read settings
     - Start either a direct connection or the post-quantum key negotiation process, depending on settings.
     */
    private func tryStart(
        nextRelays: NextRelays,
        reason: ActorReconnectReason = .userInitiated
    ) async throws {
        let settings: Settings = try settingsReader.read()
        try await self.applyNetworkSettingsIfNeeded(settings: settings)

        if settings.quantumResistance.isEnabled || settings.daita.daitaState.isEnabled {
            try await tryStartEphemeralPeerNegotiation(withSettings: settings, nextRelays: nextRelays, reason: reason)
        } else {
            try await tryStartConnection(withSettings: settings, nextRelays: nextRelays, reason: reason)
        }
    }

    private func applyNetworkSettingsIfNeeded(settings: Settings) async throws {
        let tunnelSettings = settings.interfaceSettings()
        if self.lastAppliedTunnelSettings != tunnelSettings {
            try await tunnelAdapter.apply(settings: tunnelSettings)
            self.lastAppliedTunnelSettings = tunnelSettings
        }
    }

    /**
     Attempt to start a direct (non-quantum) connection to the tunnel by performing the following steps:

     - Determine target state, it can either be `.connecting` or `.reconnecting`. (See `TargetStateForReconnect`)
     - Bail if target state cannot be determined. That means that the actor is past the point when it could logically connect or reconnect, i.e it can already be in
     `.disconnecting` state.
     - Configure tunnel adapter.
     - Start tunnel monitor.
     - Reactivate default path observation (disabled when configuring tunnel adapter)

     - Parameters:
     - nextRelays: which relays should be selected next.
     - reason: reason for reconnect
     */
    private func tryStartConnection(
        withSettings settings: Settings,
        nextRelays: NextRelays,
        reason: ActorReconnectReason
    ) async throws {
        guard let connectionState = try obfuscateConnection(nextRelays: nextRelays, settings: settings, reason: reason),
            let targetState = state.targetStateForReconnect
        else { return }
        let configuration = try ConnectionConfigurationBuilder(
            type: .normal,
            settings: settings,
            connectionData: connectionState
        ).make()

        let entryConfiguration = configuration.entryConfiguration
        let exitConfiguration = configuration.exitConfiguration

        // Daita parameters are gotten from an ephemeral peer
        try await tunnelAdapter.startMultihop(
            entryConfiguration: entryConfiguration,
            exitConfiguration: exitConfiguration,
            daita: nil
        )

        // Resume tunnel monitoring and use IPv4 gateway as a probe address.
        tunnelMonitor.start(probeAddress: connectionState.selectedRelays.exit.endpoint.ipv4Gateway)

        switch targetState {
        case .connecting:
            state = .connecting(connectionState)
        case .reconnecting:
            state = .reconnecting(connectionState)
        }
    }

    /**
     Derive `ConnectionState` from current `state` updating it with new relays and settings.

     - Parameters:
     - nextRelays: relay preference that should be used when selecting next relays.
     - settings: current settings
     - reason: reason for reconnect

     - Returns: New connection state or `nil` if current state is at or past `.disconnecting` phase.
     */
    internal func makeConnectionState(
        nextRelays: NextRelays,
        settings: Settings,
        reason: ActorReconnectReason
    ) throws -> State.ConnectionData? {
        var keyPolicy: State.KeyPolicy = .useCurrent
        var networkReachability = defaultPathObserver.currentPathStatus.networkReachability
        var lastKeyRotation: Date?

        let callRelaySelector = { [self] maybeCurrentRelays, connectionCount in
            try self.selectRelays(
                nextRelays: nextRelays,
                relayConstraints: settings.relayConstraints,
                currentRelays: maybeCurrentRelays,
                tunnelSettings: settings.tunnelSettings,
                connectionAttemptCount: connectionCount
            )
        }

        switch state {
        // Handle ephemeral peers separately as they don't interfere with either the `.connecting` or `.reconnecting` states.
        case var .negotiatingEphemeralPeer(connectionState, _):
            if reason == .connectionLoss {
                connectionState.incrementAttemptCount()
            }
            let selectedRelays = try callRelaySelector(
                connectionState.selectedRelays,
                connectionState.connectionAttemptCount
            )
            let connectedRelay = selectedRelays.entry ?? selectedRelays.exit
            connectionState.selectedRelays = selectedRelays
            connectionState.relayConstraints = settings.relayConstraints
            connectionState.connectedEndpoint = connectedRelay.endpoint
            connectionState.remotePort = connectedRelay.endpoint.ipv4Relay.port
            connectionState.obfuscationMethod = selectedRelays.obfuscation

            return connectionState
        case var .connecting(connectionState), var .reconnecting(connectionState):
            if reason == .connectionLoss {
                connectionState.incrementAttemptCount()
            }
            fallthrough
        case var .connected(connectionState):
            let selectedRelays = try callRelaySelector(
                connectionState.selectedRelays,
                connectionState.connectionAttemptCount
            )
            let connectedRelay = selectedRelays.entry ?? selectedRelays.exit
            connectionState.selectedRelays = selectedRelays
            connectionState.relayConstraints = settings.relayConstraints
            connectionState.currentKey = settings.privateKey
            connectionState.connectedEndpoint = connectedRelay.endpoint
            connectionState.remotePort = connectedRelay.endpoint.ipv4Relay.port
            connectionState.obfuscationMethod = selectedRelays.obfuscation
            return connectionState
        case let .error(blockedState):
            keyPolicy = blockedState.keyPolicy
            lastKeyRotation = blockedState.lastKeyRotation
            networkReachability = blockedState.networkReachability
            fallthrough
        case .initial:
            let selectedRelays = try callRelaySelector(nil, 0)
            let connectedRelay = selectedRelays.entry ?? selectedRelays.exit
            return State.ConnectionData(
                selectedRelays: selectedRelays,
                relayConstraints: settings.relayConstraints,
                currentKey: settings.privateKey,
                keyPolicy: keyPolicy,
                networkReachability: networkReachability,
                connectionAttemptCount: 0,
                lastKeyRotation: lastKeyRotation,
                connectedEndpoint: connectedRelay.endpoint,
                transportLayer: .udp,
                remotePort: connectedRelay.endpoint.ipv4Relay.port,
                isPostQuantum: settings.quantumResistance.isEnabled,
                isDaitaEnabled: settings.daita.daitaState.isEnabled,
                obfuscationMethod: selectedRelays.obfuscation
            )
        case .disconnecting, .disconnected:
            return nil
        }
    }

    internal func activeKey(from state: State.ConnectionData, in settings: Settings) -> PrivateKey {
        switch state.keyPolicy {
        case .useCurrent:
            settings.privateKey
        case let .usePrior(priorKey, _):
            priorKey
        }
    }

    internal func obfuscateConnection(
        nextRelays: NextRelays,
        settings: Settings,
        reason: ActorReconnectReason
    ) throws -> State.ConnectionData? {
        guard let connectionState = try makeConnectionState(nextRelays: nextRelays, settings: settings, reason: reason)
        else { return nil }

        let obfuscated = protocolObfuscator.obfuscate(
            connectionState.connectedEndpoint,
            relayFeatures: connectionState.selectedRelays.entry?.features
                ?? connectionState.selectedRelays.exit
                .features, obfuscationMethod: connectionState.obfuscationMethod
        )
        let transportLayer = protocolObfuscator.transportLayer.map { $0 } ?? .udp

        return State.ConnectionData(
            selectedRelays: connectionState.selectedRelays,
            relayConstraints: connectionState.relayConstraints,
            currentKey: settings.privateKey,
            keyPolicy: connectionState.keyPolicy,
            networkReachability: connectionState.networkReachability,
            connectionAttemptCount: connectionState.connectionAttemptCount,
            lastKeyRotation: connectionState.lastKeyRotation,
            connectedEndpoint: obfuscated.endpoint,
            transportLayer: transportLayer,
            remotePort: protocolObfuscator.remotePort,
            isPostQuantum: settings.quantumResistance.isEnabled,
            isDaitaEnabled: settings.daita.daitaState.isEnabled,
            obfuscationMethod: obfuscated.method
        )
    }

    /**
     Select next relay to connect to based on `NextRelays` and other input parameters.

     - Parameters:
     - nextRelays: next relays to connect to.
     - relayConstraints: relay constraints.
     - currentRelays: currently selected relays.
     - connectionAttemptCount: number of failed connection attempts so far.

     - Returns: selector result that contains the credentials of the next relays that the tunnel should connect to.
     */
    private func selectRelays(
        nextRelays: NextRelays,
        relayConstraints: RelayConstraints,
        currentRelays: SelectedRelays?,
        tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) throws -> SelectedRelays {
        switch nextRelays {
        case .current:
            if let currentRelays {
                return currentRelays
            } else {
                // Fallthrough to .random when current relays are not set.
                fallthrough
            }

        case .random:
            return try relaySelector.selectRelays(
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            )

        case let .preSelected(selectedRelays):
            return selectedRelays
        }
    }
}

extension PacketTunnelActor: PacketTunnelActorProtocol {}
