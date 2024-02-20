//
//  PacketTunnelActor.swift
//  PacketTunnel
//
//  Created by pronebird on 30/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import NetworkExtension
import TunnelObfuscation
import WireGuardKitTypes

/**
 Packet tunnel state machine implemented as an actor.

 - Actor receives commands for execution over the `CommandChannel`.

 - Commands are consumed in a detached task via for-await loop over the channel. Each command, once received, is executed in its entirety before the next
   command is processed. See the implementation of `consumeCommands()` which is the central task dispatcher inside of actor.

 - Most of calls that actor performs suspend for a very short amount of time. `CommandChannel` proactively discards unwanted tasks as they arrive to prevent
   future execution, such as repeating commands to reconnect are coalesced and all commands prior to stop are discarded entirely as the outcome would be the
   same anyway.
 */
public actor PacketTunnelActor {
    var state: State = .initial {
        didSet {
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
    let relaySelector: RelaySelectorProtocol
    let settingsReader: SettingsReaderProtocol
    let protocolObfuscator: ProtocolObfuscation

    nonisolated let commandChannel = CommandChannel()

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

        consumeCommands(channel: commandChannel)
    }

    deinit {
        commandChannel.finish()
    }

    /**
     Spawn a detached task that consumes commands from the channel indefinitely until the channel is closed.
     Commands are processed one at a time, so no suspensions should affect the order of execution and thus guarantee transactional execution.

     - Parameter channel: command channel.
     */
    private nonisolated func consumeCommands(channel: CommandChannel) {
        Task.detached { [weak self] in
            for await command in channel {
                guard let self else { return }

                self.logger.debug("Received command: \(command.logFormat())")

                switch command {
                case let .start(options):
                    await start(options: options)

                case .stop:
                    await stop()

                case let .reconnect(nextRelay, reason):
                    await reconnect(to: nextRelay, reason: reason)

                case let .error(reason):
                    await setErrorStateInternal(with: reason)

                case let .notifyKeyRotated(date):
                    await cacheActiveKey(lastKeyRotation: date)

                case .switchKey:
                    await switchToCurrentKey()

                case let .monitorEvent(event):
                    await handleMonitorEvent(event)

                case let .networkReachability(defaultPath):
                    await handleDefaultPathChange(defaultPath)
                case let .negotiatePostQuantumKey(StartOptions):
                    await negotiatePostQuantumKeyExchange(StartOptions)
                }
            }
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
        // TODO: Make sure that we are **Not** in the initial state anymore if PQ key exchange happened
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

    private func negotiatePostQuantumKeyExchange(_ options: StartOptions, nextRelay: NextRelay = .current) async {
        // TODO: Should this be the same path as in a reconnection attempt ?
        guard case .initial = state else { return }
        do {
            let settings: Settings = try settingsReader.read()
            // `.automatic` is the same as `off` by default for now
            if settings.tunnelQuantumResistance == .on {
                // Generate the configuration for PQ key exchange
                let postQuantumConfiguration = ConfigurationBuilder(
                    privateKey: settings.privateKey,
                    interfaceAddresses: settings.interfaceAddresses, allowedIPs: [
                        IPAddressRange(from: "10.64.0.1/32")!,
                    ]
                )
                let tunnelAdapterConfiguration = try postQuantumConfiguration.makeConfiguration()

                // Start the adapter with the configuration
                try await tunnelAdapter.startPostQuantumKeyExchange(configuration: tunnelAdapterConfiguration)

                // Do the GRPC call from `talpid-tunnel-config-client`
                if await negotiatePostQuantumKey() {
                    commandChannel.send(.start(options))
                } else {
                    // TODO: Insert some kind of delay before retrying probably, or let the natural delay from the GRPC failure dictate how long to wait
                    // TODO: Change the selected relay here, reuse the same logic as a normal connection
                    await negotiatePostQuantumKeyExchange(options, nextRelay: .random)
                }
            }
        } catch {
            // TODO: probably enter error state here
            // TODO: OR just bounce between relays until we manage to connect to a relay ?
        }
    }

    func negotiatePostQuantumKey() async -> Bool {
        // Do the GRPC call from `talpid-tunnel-config-client`
        true
    }

    /// Stop the tunnel.
    private func stop() async {
        switch state {
        case let .connected(connState), let .connecting(connState), let .reconnecting(connState):
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
    private func reconnect(to nextRelay: NextRelay, reason: ReconnectReason) async {
        do {
            switch state {
            case .connecting, .connected, .reconnecting, .error:
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
     Attempt to start the tunnel by performing the following steps:

     - Read settings.
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
    private func tryStart(
        nextRelay: NextRelay = .random,
        reason: ReconnectReason = .userInitiated
    ) async throws {
        let settings: Settings = try settingsReader.read()

        guard let connectionState = try obfuscateConnection(nextRelay: nextRelay, settings: settings, reason: reason),
              let targetState = state.targetStateForReconnect else { return }

        let activeKey: PrivateKey
        switch connectionState.keyPolicy {
        case .useCurrent:
            activeKey = settings.privateKey
        case let .usePrior(priorKey, _):
            activeKey = priorKey
        }

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
    private func makeConnectionState(
        nextRelay: NextRelay,
        settings: Settings,
        reason: ReconnectReason
    ) throws -> ConnectionState? {
        var keyPolicy: KeyPolicy = .useCurrent
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
        return ConnectionState(
            selectedRelay: selectedRelay,
            relayConstraints: settings.relayConstraints,
            currentKey: settings.privateKey,
            keyPolicy: keyPolicy,
            networkReachability: networkReachability,
            connectionAttemptCount: 0,
            lastKeyRotation: lastKeyRotation,
            connectedEndpoint: selectedRelay.endpoint,
            transportLayer: .udp,
            remotePort: selectedRelay.endpoint.ipv4Relay.port
        )
    }

    private func obfuscateConnection(
        nextRelay: NextRelay,
        settings: Settings,
        reason: ReconnectReason
    ) throws -> ConnectionState? {
        guard let connectionState = try makeConnectionState(nextRelay: nextRelay, settings: settings, reason: reason)
        else { return nil }

        let obfuscatedEndpoint = protocolObfuscator.obfuscate(
            connectionState.selectedRelay.endpoint,
            settings: settings,
            retryAttempts: connectionState.selectedRelay.retryAttempts
        )

        let transportLayer = protocolObfuscator.transportLayer.map { $0 } ?? .udp
        return ConnectionState(
            selectedRelay: connectionState.selectedRelay,
            relayConstraints: connectionState.relayConstraints,
            currentKey: settings.privateKey,
            keyPolicy: connectionState.keyPolicy,
            networkReachability: connectionState.networkReachability,
            connectionAttemptCount: connectionState.connectionAttemptCount,
            lastKeyRotation: connectionState.lastKeyRotation,
            connectedEndpoint: obfuscatedEndpoint,
            transportLayer: transportLayer,
            remotePort: protocolObfuscator.remotePort
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
            return try relaySelector.selectRelay(
                with: relayConstraints,
                connectionAttemptFailureCount: connectionAttemptCount
            )

        case let .preSelected(selectedRelay):
            return selectedRelay
        }
    }
}

extension PacketTunnelActor: PacketTunnelActorProtocol {}
// swiftlint:disable:this file_length
