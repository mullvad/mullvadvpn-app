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

 ## State consistency

 All public methods, that mutate `state`, use `TaskQueue` to prevent re-entrancy and interlacing issues. Regardless how many suspensions the task
 scheduled on task queue may have, it will execute in its entirety before passing control to the next.

 Task queue also enables actor to coalesce repeating calls, and cancel executing tasks that no longer need to continue.

 ## Internal task queue

 Developers working on actor must avoid circular dependencies between operations added to `TaskQueue`. The common mistake would be to add a new task
 to the queue and then await for it to complete, while running in the context of another task executing on the queue. This scenario would lead to deadlock, because
 the queue organizes operations in a linked list, so that each subsequent operation waits for the one before to complete.

 Most of private methods are documented to outline the expected execution context.
 */
public actor PacketTunnelActor {
    @Published internal(set) public var state: State = .initial {
        didSet {
            logger.debug("\(state.logFormat())")
        }
    }

    let logger = Logger(label: "PacketTunnelActor")
    let taskQueue = TaskQueue()

    let timings: PacketTunnelActorTimings
    let tunnelAdapter: TunnelAdapterProtocol
    let tunnelMonitor: TunnelMonitorProtocol
    let defaultPathObserver: DefaultPathObserverProtocol
    let blockedStateErrorMapper: BlockedStateErrorMapperProtocol
    let relaySelector: RelaySelectorProtocol
    let settingsReader: SettingsReaderProtocol

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
    }
}

// MARK: - Public: Calls serialized on task queue

extension PacketTunnelActor {
    /**
     Start the tunnel and wait until connected.

     - Can only be called once, all subsequent attempts are ignored. Use `reconnect()` if you wish to change relay.
     - Can be cancelled by a subsequent call to `stop()`. In that case the original invocation of `start()` returns once actor moved to disconnected state.

     - Important: Internal implementation must not call this method from operation executing on `TaskQueue`.

     - Parameter options: start options produced by packet tunnel
     */
    public func start(options: StartOptions) async throws {
        try await taskQueue.add(kind: .start) { [self] in
            guard case .initial = state else { return }

            logger.debug("\(options.logFormat())")

            // Start observing default network path to determine network reachability.
            startDefaultPathObserver()

            // Assign a closure receiving tunnel monitor events.
            setTunnelMonitorEventHandler()

            do {
                try await tryStart(nextRelay: options.selectorResult.map { .preSelected($0) } ?? .random)
            } catch {
                try Task.checkCancellation()

                logger.error(error: error, message: "Failed to start the tunnel.")

                await setErrorStateInner(with: error)
            }
        }

        /*
         Wait until the state moved to `.connected`.

         Note that this `await` call happens outside of `taskQueue` to avoid blocking it, so that calls to `reconnect()`
         can execute freely.

         This is mostly done so that packet tunnel provider could return from `startTunnel()` only once the tunnel is
         fully connected which should transition it from `NEVPNStatus.connecting` → `NEVPNStatus.connected`.
         */
        await waitUntilConnected()
    }

    /**
     Stop the tunnel.

     - Important: Internal implementation must not call this method from operation executing on `TaskQueue`.

     Calling this method cancels all pending or executing tasks.
     */
    public func stop() async {
        try? await taskQueue.add(kind: .stop) { [self] in
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
    }

    /**
     Reconnect the tunnel to new relay.

     - Important: Internal implementation must not call this method from operation executing on `TaskQueue`.

     - Parameter nextRelay: next relay to connect to.
     */
    public func reconnect(to nextRelay: NextRelay) async throws {
        try await enqueueReconnect(to: nextRelay, shouldStopTunnelMonitor: true)
    }

    // MARK: - Private: Tunnel management

    /**
     Attempt to start the tunnel by performing the following steps:

     - Read settings.
     - Determine target state, it can either be `.connecting` or `.reconnecting`. (See `TargetStateForReconnect`)
     - Bail if target state cannot be determined. That means that the actor is past the point when it could logically connect or reconnect, i.e it can already be in
     `.disconnecting` state.
     - Configure tunnel adapter.
     - Start tunnel monitor.
     - Reactivate default path observation (disabled when configuring tunnel adapter)

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue`.

     - Parameter nextRelay: which relay should be selected next.
     */
    private func tryStart(nextRelay: NextRelay = .random) async throws {
        func makeConnectionState(settings: Settings) throws -> ConnectionState? {
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

        let settings: Settings = try settingsReader.read()
        guard let connectionState = try makeConnectionState(settings: settings),
              let targetState = state.targetStateForReconnect else { return }

        switch targetState {
        case .connecting:
            state = .connecting(connectionState)
        case .reconnecting:
            state = .reconnecting(connectionState)
        }

        let endpoint = connectionState.selectedRelay.endpoint
        let configurationBuilder = ConfigurationBuilder(
            privateKey: connectionState.activeKey,
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
     Schedule reconnection attempt on `TaskQueue`.

     - Important: Internal implementation must not call this method from operation executing on `TaskQueue`.
     */
    private func enqueueReconnect(to nextRelay: NextRelay, shouldStopTunnelMonitor: Bool) async throws {
        try await taskQueue.add(kind: .reconnect) { [self] in
            try await reconnectInner(to: nextRelay, shouldStopTunnelMonitor: shouldStopTunnelMonitor)
        }
    }

    /**
     Perform a reconnection attempt. Enters error state on failure.

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue`.

     - Parameters:
     - nextRelay: next relay to connect to
     - shouldStopTunnelMonitor: whether tunnel monitor should be stopped
     */
    private func reconnectInner(to nextRelay: NextRelay, shouldStopTunnelMonitor: Bool) async throws {
        // Sleep a bit to provide a debounce in case we get a barrage of calls to reconnect.
        // Task.sleep() throws CancellationError if the task is cancelled.
        try await Task.sleep(duration: timings.reconnectDebounce)

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
            try Task.checkCancellation()

            logger.error(error: error, message: "Failed to reconnect the tunnel.")

            await setErrorStateInner(with: error)
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
        currentRelay: RelaySelectorResult?,
        connectionAttemptCount: UInt
    ) throws -> RelaySelectorResult {
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

        case let .preSelected(selectorResult):
            return selectorResult
        }
    }
}
