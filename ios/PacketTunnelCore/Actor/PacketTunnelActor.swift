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
    var tunnelMonitorTask: Task<Void, Never>?

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

    /**
     Spawn a detached task that consumes events from the channel indefinitely until the channel is closed.
     Events are processed one at a time, so no suspensions should affect the order of execution and thus guarantee transactional execution.

     - Parameter channel: event channel.
     */
    private nonisolated func consumeEvents(channel: EventChannel) {
        Task.detached { [weak self] in
            for await event in channel {
                guard let self else { return }

                self.logger.debug("Received event: \(event.logFormat())")

                let effects = await self.runReducer(event)

                for effect in effects {
                    await executeEffect(effect)
                }
            }
        }
    }

    // swiftlint:disable:next function_body_length
    func executeEffect(_ effect: Effect) async {
        switch effect {
        case .startDefaultPathObserver:
            startDefaultPathObserver()
        case .stopDefaultPathObserver:
            stopDefaultPathObserver()
        case .startTunnelMonitor:
            await listenForTunnelMonitorEvents()
        case .stopTunnelMonitor:
            await tunnelMonitor.stop()
        case let .updateTunnelMonitorPath(networkPath):
            await handleDefaultPathChange(networkPath)
        case let .startConnection(nextRelays):
            do {
                try await tryStart(nextRelays: nextRelays)
            } catch {
                logger.error(error: error, message: "Failed to start the tunnel.")
                await setErrorStateInternal(with: error)
            }
        case let .restartConnection(nextRelays, reason):
            do {
                try await tryStart(nextRelays: nextRelays, reason: reason)
            } catch {
                logger.error(error: error, message: "Failed to reconnect the tunnel.")
                await setErrorStateInternal(with: error)
            }
        case let .reconnect(nextRelay):
            eventChannel.send(.reconnect(nextRelay))
        case .stopTunnelAdapter:
            do {
                try await tunnelAdapter.stop()
            } catch {
                logger.error(error: error, message: "Failed to stop adapter.")
            }
            state = .disconnected
        case let .configureForErrorState(reason):
            await setErrorStateInternal(with: reason)
        case let .cacheActiveKey(lastKeyRotation):
            cacheActiveKey(lastKeyRotation: lastKeyRotation)
        case let .reconfigureForEphemeralPeer(configuration, configurationSemaphore):
            do {
                try await updateEphemeralPeerNegotiationState(configuration: configuration)
            } catch {
                logger.error(error: error, message: "Failed to reconfigure tunnel after each hop negotiation.")
                await setErrorStateInternal(with: error)
            }
            configurationSemaphore.send()
        case .connectWithEphemeralPeer:
            await connectWithEphemeralPeer()
        case .setDisconnectedState:
            self.state = .disconnected
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

        // Start observing default network path to determine network reachability.
        startDefaultPathObserver()

        // Assign a closure receiving tunnel monitor events.
        await listenForTunnelMonitorEvents()

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
            await tunnelMonitor.stop()

            // Fallthrough to stop adapter and shift to `.disconnected` state.
            fallthrough

        case .error:
            stopDefaultPathObserver()

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
     Reconnect tunnel to new relays. Enters error state on failure.

     - Parameters:
     - nextRelay: next relays to connect to
     - reason: reason for reconnect
     */
    private func reconnect(to nextRelays: NextRelays, reason: ActorReconnectReason) async {
        do {
            switch state {
            // There is no connection monitoring going on when exchanging keys.
            // The procedure starts from scratch for each reconnection attempts.
            case .connecting, .connected, .reconnecting, .error, .negotiatingEphemeralPeer:
                switch reason {
                case .connectionLoss:
                    // Tunnel monitor is already paused at this point. Avoid calling stop() to prevent the reset of
                    // internal state
                    break
                case .userInitiated:
                    await tunnelMonitor.stop()
                }

                try await tryStart(nextRelays: nextRelays, reason: reason)

            case .disconnected, .disconnecting, .initial:
                break
            }
        } catch {
            logger.error(error: error, message: "Failed to reconnect the tunnel.")

            await setErrorStateInternal(with: error)
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

        if settings.quantumResistance.isEnabled || settings.daita.daitaState.isEnabled {
            try await tryStartEphemeralPeerNegotiation(withSettings: settings, nextRelays: nextRelays, reason: reason)
        } else {
            try await tryStartConnection(withSettings: settings, nextRelays: nextRelays, reason: reason)
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
              let targetState = state.targetStateForReconnect else { return }
        let configuration = try ConnectionConfigurationBuilder(
            type: .normal,
            settings: settings,
            connectionData: connectionState
        ).make()

        /*
         Stop default path observer while updating WireGuard configuration since it will call the system method
         `NEPacketTunnelProvider.setTunnelNetworkSettings()` which may cause active interfaces to go down making it look
         like network connectivity is not available, but only for a brief moment.
         */
        stopDefaultPathObserver()

        defer {
            // Restart default path observer and notify the observer with the current path that might have changed while
            // path observer was paused.
            startDefaultPathObserver()
        }

        // Daita parameters are gotten from an ephemeral peer
        try await tunnelAdapter.startMultihop(
            entryConfiguration: configuration.entryConfiguration,
            exitConfiguration: configuration.exitConfiguration,
            daita: nil
        )

        // Resume tunnel monitoring and use IPv4 gateway as a probe address.
        await tunnelMonitor.start(probeAddress: connectionState.selectedRelays.exit.endpoint.ipv4Gateway)

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
    // swiftlint:disable:next function_body_length
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
                isDaitaEnabled: settings.daita.daitaState.isEnabled
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

        let obfuscatedEndpoint = protocolObfuscator.obfuscate(
            connectionState.connectedEndpoint,
            settings: settings.tunnelSettings,
            retryAttempts: connectionState.selectedRelays.retryAttempt
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
            connectedEndpoint: obfuscatedEndpoint,
            transportLayer: transportLayer,
            remotePort: protocolObfuscator.remotePort,
            isPostQuantum: settings.quantumResistance.isEnabled,
            isDaitaEnabled: settings.daita.daitaState.isEnabled
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

// swiftlint:disable:this file_length
