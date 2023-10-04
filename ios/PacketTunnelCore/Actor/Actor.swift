//
//  Actor.swift
//  PacketTunnel
//
//  Created by pronebird on 30/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import NetworkExtension
import struct RelaySelector.RelaySelectorResult
import class WireGuardKitTypes.PrivateKey

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
    @Published internal(set) public var state: State = .initial {
        didSet {
            logger.debug("\(state.logFormat())")
        }
    }

    let logger = Logger(label: "PacketTunnelActor")

    let timings: PacketTunnelActorTimings
    let tunnelAdapter: TunnelAdapterProtocol
    let tunnelMonitor: TunnelMonitorProtocol
    let defaultPathObserver: DefaultPathObserverProtocol
    let blockedStateErrorMapper: BlockedStateErrorMapperProtocol
    let relaySelector: RelaySelectorProtocol
    let settingsReader: SettingsReaderProtocol

    nonisolated let commandChannel = CommandChannel()

    public init(
        timings: PacketTunnelActorTimings,
        tunnelAdapter: TunnelAdapterProtocol,
        tunnelMonitor: TunnelMonitorProtocol,
        defaultPathObserver: DefaultPathObserverProtocol,
        blockedStateErrorMapper: BlockedStateErrorMapperProtocol,
        relaySelector: RelaySelectorProtocol,
        settingsReader: SettingsReaderProtocol
    ) {
        self.timings = timings
        self.tunnelAdapter = tunnelAdapter
        self.tunnelMonitor = tunnelMonitor
        self.defaultPathObserver = defaultPathObserver
        self.blockedStateErrorMapper = blockedStateErrorMapper
        self.relaySelector = relaySelector
        self.settingsReader = settingsReader

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

                case let .reconnect(nextRelay, stopTunnelMonitor):
                    await reconnect(to: nextRelay, shouldStopTunnelMonitor: stopTunnelMonitor)

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
         - shouldStopTunnelMonitor: whether tunnel monitor should be stopped
     */
    private func reconnect(to nextRelay: NextRelay, shouldStopTunnelMonitor: Bool) async {
        do {
            switch state {
            case .connecting, .connected, .reconnecting, .error:
                if shouldStopTunnelMonitor {
                    tunnelMonitor.stop()
                }
                try await tryStart(nextRelay: nextRelay)

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

     - Parameter nextRelay: which relay should be selected next.
     */
    private func tryStart(nextRelay: NextRelay = .random) async throws {
        let settings: Settings = try settingsReader.read()

        guard let connectionState = try makeConnectionState(nextRelay: nextRelay, settings: settings),
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

        let endpoint = connectionState.selectedRelay.endpoint
        let configurationBuilder = ConfigurationBuilder(
            privateKey: activeKey,
            interfaceAddresses: settings.interfaceAddresses,
            dns: settings.dnsServers,
            endpoint: endpoint
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
        tunnelMonitor.start(probeAddress: endpoint.ipv4Gateway)
    }

    /**
     Derive `ConnectionState` from current `state` updating it with new relay and settings.

     - Parameters:
         - nextRelay: relay preference that should be used when selecting next relay.
         - settings: current settings

     - Returns: New connection state or `nil` if current state is at or past `.disconnecting` phase.
     */
    private func makeConnectionState(nextRelay: NextRelay, settings: Settings) throws -> ConnectionState? {
        let relayConstraints = settings.relayConstraints
        let privateKey = settings.privateKey

        switch state {
        case .initial:
            return ConnectionState(
                selectedRelay: try selectRelay(
                    nextRelay: nextRelay,
                    relayConstraints: relayConstraints,
                    currentRelay: nil,
                    connectionAttemptCount: 0
                ),
                relayConstraints: relayConstraints,
                currentKey: privateKey,
                keyPolicy: .useCurrent,
                networkReachability: defaultPathObserver.defaultPath?.networkReachability ?? .undetermined,
                connectionAttemptCount: 0
            )

        case var .connecting(connState), var .connected(connState), var .reconnecting(connState):
            connState.selectedRelay = try selectRelay(
                nextRelay: nextRelay,
                relayConstraints: relayConstraints,
                currentRelay: connState.selectedRelay,
                connectionAttemptCount: connState.connectionAttemptCount
            )
            connState.relayConstraints = relayConstraints
            connState.currentKey = privateKey

            return connState

        case let .error(blockedState):
            return ConnectionState(
                selectedRelay: try selectRelay(
                    nextRelay: nextRelay,
                    relayConstraints: relayConstraints,
                    currentRelay: nil,
                    connectionAttemptCount: 0
                ),
                relayConstraints: relayConstraints,
                currentKey: privateKey,
                keyPolicy: blockedState.keyPolicy,
                networkReachability: blockedState.networkReachability,
                connectionAttemptCount: 0,
                lastKeyRotation: blockedState.lastKeyRotation
            )

        case .disconnecting, .disconnected:
            return nil
        }
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

extension RelaySelectorResult {
    func asSelectedRelay() -> SelectedRelay {
        return SelectedRelay(endpoint: endpoint, hostname: relay.hostname, location: location)
    }
}
