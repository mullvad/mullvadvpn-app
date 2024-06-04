//
//  PacketTunnelActor.swift
//  PacketTunnel
//
//  Created by pronebird on 30/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import NetworkExtension
import TunnelObfuscation
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

    let logger = Logger(label: "PacketTunnelActor")

    let timings: PacketTunnelActorTimings
    let tunnelAdapter: TunnelAdapterProtocol
    let tunnelMonitor: TunnelMonitorProtocol
    let defaultPathObserver: DefaultPathObserverProtocol
    let blockedStateErrorMapper: BlockedStateErrorMapperProtocol
    public let relaySelector: RelaySelectorProtocol
    let settingsReader: SettingsReaderProtocol
    let protocolObfuscator: ProtocolObfuscation

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

    func executeEffect(_ effect: Effect) async {
        switch effect {
        case .startDefaultPathObserver:
            startDefaultPathObserver()
        case .stopDefaultPathObserver:
            stopDefaultPathObserver()
        case .startTunnelMonitor:
            setTunnelMonitorEventHandler()
        case .stopTunnelMonitor:
            tunnelMonitor.stop()
        case let .updateTunnelMonitorPath(networkPath):
            handleDefaultPathChange(networkPath)
        case let .startConnection(nextRelay):
            do {
                try await tryStart(nextRelay: nextRelay)
            } catch {
                logger.error(error: error, message: "Failed to start the tunnel.")
                await setErrorStateInternal(with: error)
            }
        case let .restartConnection(nextRelay, reason):
            do {
                try await tryStart(nextRelay: nextRelay, reason: reason)
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
        case let .postQuantumConnect(key, privateKey: privateKey):
            await postQuantumConnect(with: key, privateKey: privateKey)
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
        setTunnelMonitorEventHandler()

        do {
            try await tryStart(nextRelay: options.selectedRelay.map { .preSelected($0) } ?? .random)
        } catch {
            logger.error(error: error, message: "Failed to start the tunnel.")

            await setErrorStateInternal(with: error)
        }
    }

    /// Stop the tunnel.
    private func stop() async {
        switch state {
        case let .connected(connState), let .connecting(connState), let .reconnecting(connState),
             let .negotiatingPostQuantumKey(connState, _):
            state = .disconnecting(connState)
            tunnelMonitor.stop()

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
     Reconnect tunnel to new relay. Enters error state on failure.

     - Parameters:
         - nextRelay: next relay to connect to
         - reason: reason for reconnect
     */
    private func reconnect(to nextRelay: NextRelay, reason: ActorReconnectReason) async {
        do {
            switch state {
            // There is no connection monitoring going on when exchanging keys.
            // The procedure starts from scratch for each reconnection attempts.
            case .connecting, .connected, .reconnecting, .error, .negotiatingPostQuantumKey:
                switch reason {
                case .connectionLoss:
                    // Tunnel monitor is already paused at this point. Avoid calling stop() to prevent the reset of
                    // internal state
                    break
                case .userInitiated:
                    tunnelMonitor.stop()
                }

                try await tryStart(nextRelay: nextRelay, reason: reason)

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
        nextRelay: NextRelay,
        reason: ActorReconnectReason = .userInitiated
    ) async throws {
        let settings: Settings = try settingsReader.read()

        if settings.quantumResistance.isEnabled {
            try await tryStartPostQuantumNegotiation(withSettings: settings, nextRelay: nextRelay, reason: reason)
        } else {
            try await tryStartConnection(withSettings: settings, nextRelay: nextRelay, reason: reason)
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
         - nextRelay: which relay should be selected next.
         - reason: reason for reconnect
     */
    private func tryStartConnection(
        withSettings settings: Settings,
        nextRelay: NextRelay,
        reason: ActorReconnectReason
    ) async throws {
        guard let connectionState = try obfuscateConnection(nextRelay: nextRelay, settings: settings, reason: reason),
              let targetState = state.targetStateForReconnect else { return }

        let activeKey = activeKey(from: connectionState, in: settings)

        switch targetState {
        case .connecting:
            state = .connecting(connectionState)
        case .reconnecting:
            state = .reconnecting(connectionState)
        }

        let configurationBuilder = ConfigurationBuilder(
            privateKey: activeKey,
            interfaceAddresses: settings.interfaceAddresses,
            dns: settings.dnsServers,
            endpoint: connectionState.connectedEndpoint,
            allowedIPs: [
                IPAddressRange(from: "0.0.0.0/0")!,
                IPAddressRange(from: "::/0")!,
            ]
        )

        /*
         Stop default path observer while updating WireGuard configuration since it will call the system method
         `NEPacketTunnelProvider.setTunnelNetworkSettings()` which may cause active interfaces to go down making it look
         like network connectivity is not available, but only for a brief moment.
         */
        stopDefaultPathObserver()

        defer {
            // Restart default path observer and notify the observer with the current path that might have changed while
            // path observer was paused.
            startDefaultPathObserver(notifyObserverWithCurrentPath: true)
        }

        try await tunnelAdapter.start(configuration: configurationBuilder.makeConfiguration())

        // Resume tunnel monitoring and use IPv4 gateway as a probe address.
        tunnelMonitor.start(probeAddress: connectionState.selectedRelay.endpoint.ipv4Gateway)
    }

    /**
     Derive `ConnectionState` from current `state` updating it with new relay and settings.

     - Parameters:
         - nextRelay: relay preference that should be used when selecting next relay.
         - settings: current settings
         - reason: reason for reconnect

     - Returns: New connection state or `nil` if current state is at or past `.disconnecting` phase.
     */
    internal func makeConnectionState(
        nextRelay: NextRelay,
        settings: Settings,
        reason: ActorReconnectReason
    ) throws -> State.ConnectionData? {
        var keyPolicy: State.KeyPolicy = .useCurrent
        var networkReachability = defaultPathObserver.defaultPath?.networkReachability ?? .undetermined
        var lastKeyRotation: Date?

        let callRelaySelector = { [self] maybeCurrentRelay, connectionCount in
            try self.selectRelay(
                nextRelay: nextRelay,
                relayConstraints: settings.relayConstraints,
                currentRelay: maybeCurrentRelay,
                connectionAttemptCount: connectionCount
            )
        }

        switch state {
        case .initial:
            break
        // Handle PQ PSK separately as it doesn't interfere with either the `.connecting` or `.reconnecting` states.
        case var .negotiatingPostQuantumKey(connectionState, _):
            if reason == .connectionLoss {
                connectionState.incrementAttemptCount()
            }
            let selectedRelay = try callRelaySelector(
                connectionState.selectedRelay,
                connectionState.connectionAttemptCount
            )
            connectionState.selectedRelay = selectedRelay
            connectionState.relayConstraints = settings.relayConstraints
            return connectionState
        case var .connecting(connectionState), var .reconnecting(connectionState):
            if reason == .connectionLoss {
                connectionState.incrementAttemptCount()
            }
            fallthrough
        case var .connected(connectionState):
            let selectedRelay = try callRelaySelector(
                connectionState.selectedRelay,
                connectionState.connectionAttemptCount
            )
            connectionState.selectedRelay = selectedRelay
            connectionState.relayConstraints = settings.relayConstraints
            connectionState.currentKey = settings.privateKey
            return connectionState
        case let .error(blockedState):
            keyPolicy = blockedState.keyPolicy
            lastKeyRotation = blockedState.lastKeyRotation
            networkReachability = blockedState.networkReachability
        case .disconnecting, .disconnected:
            return nil
        }
        let selectedRelay = try callRelaySelector(nil, 0)
        return State.ConnectionData(
            selectedRelay: selectedRelay,
            relayConstraints: settings.relayConstraints,
            currentKey: settings.privateKey,
            keyPolicy: keyPolicy,
            networkReachability: networkReachability,
            connectionAttemptCount: 0,
            lastKeyRotation: lastKeyRotation,
            connectedEndpoint: selectedRelay.endpoint,
            transportLayer: .udp,
            remotePort: selectedRelay.endpoint.ipv4Relay.port,
            isPostQuantum: settings.quantumResistance.isEnabled
        )
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
        nextRelay: NextRelay,
        settings: Settings,
        reason: ActorReconnectReason
    ) throws -> State.ConnectionData? {
        guard let connectionState = try makeConnectionState(nextRelay: nextRelay, settings: settings, reason: reason)
        else { return nil }

        let obfuscatedEndpoint = protocolObfuscator.obfuscate(
            connectionState.selectedRelay.endpoint,
            settings: settings,
            retryAttempts: connectionState.selectedRelay.retryAttempts
        )

        let transportLayer = protocolObfuscator.transportLayer.map { $0 } ?? .udp
        return State.ConnectionData(
            selectedRelay: connectionState.selectedRelay,
            relayConstraints: connectionState.relayConstraints,
            currentKey: settings.privateKey,
            keyPolicy: connectionState.keyPolicy,
            networkReachability: connectionState.networkReachability,
            connectionAttemptCount: connectionState.connectionAttemptCount,
            lastKeyRotation: connectionState.lastKeyRotation,
            connectedEndpoint: obfuscatedEndpoint,
            transportLayer: transportLayer,
            remotePort: protocolObfuscator.remotePort,
            isPostQuantum: settings.quantumResistance.isEnabled
        )
    }

    /**
     Select next relay to connect to based on `NextRelay` and other input parameters.

     - Parameters:
         - nextRelay: next relay to connect to.
         - relayConstraints: relay constraints.
         - currentRelay: currently selected relay.
         - connectionAttemptCount: number of failed connection attempts so far.

     - Returns: selector result that contains the credentials of the next relay that the tunnel should connect to.
     */
    private func selectRelay(
        nextRelay: NextRelay,
        relayConstraints: RelayConstraints,
        currentRelay: SelectedRelay?,
        connectionAttemptCount: UInt
    ) throws -> SelectedRelay {
        switch nextRelay {
        case .current:
            if let currentRelay {
                return currentRelay
            } else {
                // Fallthrough to .random when current relay is not set.
                fallthrough
            }

        case .random:
            return try relaySelector.selectRelays(
                with: relayConstraints,
                connectionAttemptCount: connectionAttemptCount
            ).exit // TODO: Multihop

        case let .preSelected(selectedRelay):
            return selectedRelay
        }
    }
}

extension PacketTunnelActor: PacketTunnelActorProtocol {}
// swiftlint:disable:this file_length
