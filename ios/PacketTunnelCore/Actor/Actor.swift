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

 All public methods, that mutate `state`, use `TaskQueue` to guarantee to prevent re-entrancy and interlacing issues. Regarless how many suspensions the task
 schedule on task queue may have, it will execute in its entirety before passing control to the next.

 Task queue also enables actor to coalesce repeating calls, and cancel executing tasks that no longer need to continue.
 */
public actor PacketTunnelActor {
    @Published private(set) public var state: State = .initial {
        didSet {
            logger.debug("\(state.logFormat())")
        }
    }

    private let logger = Logger(label: "PacketTunnelActor")
    private let taskQueue = TaskQueue()

    private let timings: PacketTunnelActorTimings
    private let tunnelAdapter: TunnelAdapterProtocol
    private let tunnelMonitor: TunnelMonitorProtocol
    private let defaultPathObserver: DefaultPathObserverProtocol
    private let blockedStateErrorMapper: BlockedStateErrorMapperProtocol
    private let relaySelector: RelaySelectorProtocol
    private let settingsReader: SettingsReaderProtocol

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

    // MARK: - Public: Calls serialized on task queue

    /**
     Start the tunnel and wait until connected.

     - Can only be called once, all subsequent attempts are ignored. Use `reconnect()` if you wish to change relay.
     - Can be cancelled by a subsequent call to `stop()`. In that case the original invocation of `start()` returns once actor moved to disconnected state.

     - Parameter options: start options produced by packet tunnel
     */
    public func start(options: StartOptions) async throws {
        try await taskQueue.add(kind: .start) { [self] in
            guard case .initial = state else { return }

            logger.debug("\(options.logFormat())")

            // Start observing default network path to determine network reachability.
            startDefaultPathObserver()

            // Assign a closure receiving tunnel monitor events.
            tunnelMonitor.onEvent = { [weak self] event in
                guard let self else { return }
                Task { await self.enqueueMonitorEvent(event) }
            }

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

     - Parameter nextRelay: next relay to connect to.
     */
    public func reconnect(to nextRelay: NextRelay) async throws {
        try await enqueueReconnect(to: nextRelay, shouldStopTunnelMonitor: true)
    }

    /**
     Switch actor into error state.

     Normally actor enters error state on its own, due to unrecoverable errors. However error state can also be induced externally for example in response to
     device check indicating certain issues that actor is not able to detect on its own such as invalid account or device being revoked on backend.

     - Parameter reason: block reason
     */
    public func setErrorState(with reason: BlockedStateReason) async {
        try? await taskQueue.add(kind: .blockedState) { [self] in
            try Task.checkCancellation()
            await setErrorStateInner(with: reason)
        }
    }

    /**
     Tell actor that key rotation took place.

     When that happens the actor changes key policy to `.usePrior` caching the currently used key in associated value.

     That cached key is used by actor for some time until the new key is propagated across relays. Then it flips the key policy back to `.useCurrent` and
     reconnects the tunnel using new key.

     The `date` passed as an argument is a simple marker value passed back to UI process with actor state. This date can be used to determine when
     the main app has to re-read device state from Keychain, since there is no other mechanism to notify other process when packet tunnel mutates keychain store.
     - Parameter date: date when last key rotation took place.
     */
    public func notifyKeyRotated(date: Date? = nil) async {
        try? await taskQueue.add(kind: .keyRotated) { [self] in
            notifyKeyRotatedInner(date: date)
        }
    }

    /**
     Internal implementation that handles key rotation notification.

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue` .
     */
    private func notifyKeyRotatedInner(date: Date?) {
        func mutateConnectionState(_ connState: inout ConnectionState) -> Bool {
            switch connState.keyPolicy {
            case .useCurrent:
                connState.lastKeyRotation = date
                connState.keyPolicy = .usePrior(connState.currentKey, startKeySwitchTask())
                return true

            case .usePrior:
                // It's unlikely that we'll see subsequent key rotations happen frequently.
                return false
            }
        }

        switch state {
        case var .connecting(connState):
            if mutateConnectionState(&connState) {
                state = .connecting(connState)
            }

        case var .connected(connState):
            if mutateConnectionState(&connState) {
                state = .connected(connState)
            }

        case var .reconnecting(connState):
            if mutateConnectionState(&connState) {
                state = .reconnecting(connState)
            }

        case var .error(blockedState):
            switch blockedState.keyPolicy {
            case .useCurrent:
                if let currentKey = blockedState.currentKey {
                    blockedState.lastKeyRotation = date
                    blockedState.keyPolicy = .usePrior(currentKey, startKeySwitchTask())
                    state = .error(blockedState)
                }

            case .usePrior:
                break
            }

        case .initial, .disconnected, .disconnecting:
            break
        }
    }

    // MARK: - Public: Sleep cycle notifications

    /**
     Cients should call this method to notify actor when device becomes awake.

     `NEPacketTunnelProvider` provides the corresponding lifecycle method.
     */
    public nonisolated func onWake() {
        tunnelMonitor.onWake()
    }

    /**
     Clients should call this method to notify actor when device is about to go to sleep.

     `NEPacketTunnelProvider` provides the corresponding lifecycle method.
     */
    public func onSleep() {
        tunnelMonitor.onSleep()
    }

    // MARK: - Private: key policy

    /**
     Start a task that will wait for the new key to propagate across relays (see `PacketTunnelActorTimings.wgKeyPropagationDelay`) and then:

     1. Switch `keyPolicy` back to `.useCurrent`.
     2. Reconnect the tunnel using the new key (currently stored in device state)
     */
    private func startKeySwitchTask() -> AutoCancellingTask {
        let task = Task {
            // Wait for key to propagate across relays.
            try await Task.sleepUsingContinuousClock(for: timings.wgKeyPropagationDelay)

            // Enqueue task to change key policy.
            let isKeySwitched = try await enqueueKeySwitch()

            // Enqueue task to reconnect
            if isKeySwitched {
                try await reconnect(to: .random)
            }
        }

        return AutoCancellingTask(task)
    }

    /**
     Enqueue a task to switch key policy to `.useCurrent`.

     - Returns: `true` if the key was switched, otherwise `false`.
     */
    private func enqueueKeySwitch() async throws -> Bool {
        try await taskQueue.add(kind: .switchKey) {
            self.keySwitchInner()
        }
    }

    /**
     Switch key policy  from `.usePrior` to `.useCurrent` policy.

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue` .

     - Returns: `true` if the key policy switch happened, otherwise `false`.
     */
    private func keySwitchInner() -> Bool {
        func mutateConnectionState(_ connectionState: inout ConnectionState) -> Bool {
            switch connectionState.keyPolicy {
            case .useCurrent:
                return false

            case .usePrior:
                connectionState.keyPolicy = .useCurrent
                return true
            }
        }

        // Switch key policy to use current key.
        switch state {
        case var .connecting(connState):
            if mutateConnectionState(&connState) {
                state = .connecting(connState)
                return true
            }

        case var .connected(connState):
            if mutateConnectionState(&connState) {
                state = .connected(connState)
                return true
            }

        case var .reconnecting(connState):
            if mutateConnectionState(&connState) {
                state = .reconnecting(connState)
                return true
            }

        case var .error(blockedState):
            switch blockedState.keyPolicy {
            case .useCurrent:
                break
            case .usePrior:
                blockedState.keyPolicy = .useCurrent
                state = .error(blockedState)
                return true
            }

        case .disconnected, .disconnecting, .initial:
            break
        }
        return false
    }

    // MARK: - Network Reachability

    /**
     Start observing changes to default path.

     - Parameter notifyObserverWithCurrentPath: notifies path observer with the current path when set to `true`.
     */
    private func startDefaultPathObserver(notifyObserverWithCurrentPath: Bool = false) {
        defaultPathObserver.start { [weak self] networkPath in
            guard let self else { return }
            Task { await self.enqueueDefaultPathChange(networkPath) }
        }

        if notifyObserverWithCurrentPath, let currentPath = defaultPathObserver.defaultPath {
            Task { await self.enqueueDefaultPathChange(currentPath) }
        }
    }

    /// Stop observing changes to default path.
    private func stopDefaultPathObserver() {
        defaultPathObserver.stop()
    }

    /**
     Event handler that receives new network path and schedules it for processing on task queue to avoid interlacing with other tasks.

     - Parameter networkPath: new default path
     */
    private func enqueueDefaultPathChange(_ networkPath: NetworkPath) async {
        try? await taskQueue.add(kind: .networkReachability) { [self] in
            let newReachability = networkPath.networkReachability

            func mutateConnectionState(_ connState: inout ConnectionState) -> Bool {
                if connState.networkReachability != newReachability {
                    connState.networkReachability = newReachability
                    return true
                }
                return false
            }

            switch state {
            case var .connecting(connState):
                if mutateConnectionState(&connState) {
                    state = .connecting(connState)
                }

            case var .connected(connState):
                if mutateConnectionState(&connState) {
                    state = .connected(connState)
                }

            case var .reconnecting(connState):
                if mutateConnectionState(&connState) {
                    state = .reconnecting(connState)
                }

            case var .disconnecting(connState):
                if mutateConnectionState(&connState) {
                    state = .disconnecting(connState)
                }

            case var .error(blockedState):
                if blockedState.networkReachability != newReachability {
                    blockedState.networkReachability = newReachability
                    state = .error(blockedState)
                }

            case .initial, .disconnected:
                break
            }
        }
    }

    // MARK: - Private: Tunnel management

    /**
     Attempt to start the tunnel by performing the following steps:

     - Read settings.
     - If device is not logged in then throw an error and return
     - Determine target state, it can either be `.connecting` or `.reconnecting`. (See `TargetStateForReconnect`)
     - Bail if target state cannot be determined. That means that the actor is past the point when it could logically connect or reconnect, i.e it can already be in
     `.disconnecting` state.
     - Configure tunnel adapter.
     - Start tunnel monitor.

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

     - Important: This method must not be called from operation executing on `TaskQueue`.
     */
    private func enqueueReconnect(to nextRelay: NextRelay, shouldStopTunnelMonitor: Bool) async throws {
        try await taskQueue.add(kind: .reconnect) { [self] in
            try await reconnectInner(to: nextRelay, shouldStopTunnelMonitor: shouldStopTunnelMonitor)
        }
    }

    /**
     Perform a reconnection attempt. Enters error state on failure.

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue` .

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

    // MARK: - Private: Error state

    /**
     Transition actor to error state.

     Evaluates the error and maps it to `BlockedStateReason` before switching actor to `.error` state.

     - Parameter error: an error that occurred while starting the tunnel.
     */
    private func setErrorStateInner(with error: Error) async {
        let reason = blockedStateErrorMapper.mapError(error)

        await setErrorStateInner(with: reason)
    }

    /**
     Transition actor to error state.

     - Parameter reason: reason why the actor is entering error state.
     */
    private func setErrorStateInner(with reason: BlockedStateReason) async {
        switch state {
        case .initial:
            let blockedState = BlockedState(
                reason: reason,
                relayConstraints: nil,
                currentKey: nil,
                keyPolicy: .useCurrent,
                networkReachability: defaultPathObserver.defaultPath?.networkReachability ?? .undetermined,
                recoveryTask: startRecoveryTaskIfNeeded(reason: reason),
                priorState: state.priorState!
            )
            state = .error(blockedState)
            await configureAdapterForErrorState()

        case let .connected(connState), let .connecting(connState), let .reconnecting(connState):
            let blockedState = BlockedState(
                reason: reason,
                relayConstraints: connState.relayConstraints,
                currentKey: nil,
                keyPolicy: connState.keyPolicy,
                networkReachability: connState.networkReachability,
                priorState: state.priorState!
            )
            state = .error(blockedState)
            await configureAdapterForErrorState()

        case var .error(blockedState):
            if blockedState.reason != reason {
                blockedState.reason = reason
                state = .error(blockedState)
            }

        case .disconnecting, .disconnected:
            break
        }
    }

    /**
     Configure tunnel with empty WireGuard configuration that consumes all traffic on device and emitates the blocked state akin to the one we have on desktop
     which however utilizes firewall to achieve this.
     */
    private func configureAdapterForErrorState() async {
        do {
            let configurationBuilder = ConfigurationBuilder(
                privateKey: PrivateKey(),
                interfaceAddresses: []
            )
            try await tunnelAdapter.start(configuration: configurationBuilder.makeConfiguration())
        } catch {
            logger.error(error: error, message: "Unable to configure the tunnel for error state.")
        }
    }

    /**
     Start a task that will attempt to reconnect the tunnel periodically, but only if the tunnel can recover from error state automatically.

     See `BlockedStateReason.shouldRestartAutomatically` for more info.

     - Parameter reason: the reason why actor is entering blocked state.
     - Returns: a task that will attempt to perform periodic recovery when applicable, otherwise `nil`.
     */
    private func startRecoveryTaskIfNeeded(reason: BlockedStateReason) -> AutoCancellingTask? {
        guard reason.shouldRestartAutomatically else { return nil }

        let periodicity = timings.bootRecoveryPeriodicity
        let task = Task { [weak self] in
            while !Task.isCancelled {
                try await Task.sleepUsingContinuousClock(for: periodicity)
                try? await self?.reconnect(to: .random)
            }
        }

        return AutoCancellingTask(task)
    }

    // MARK: - Private: Connection monitoring

    /**
     Enqueue a task handling monitor event.

     - Important: This method must not be called from operation executing on `TaskQueue`.
     */
    private func enqueueMonitorEvent(_ event: TunnelMonitorEvent) async {
        try? await taskQueue.add(kind: .monitorEvent) { [self] in
            switch event {
            case .connectionEstablished:
                onEstablishConnection()
            case .connectionLost:
                await onHandleConnectionRecovery()
            }
        }
    }

    /**
     Reset connection attempt counter and update actor state to `connected` state once connection is established.

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue` .
     */
    private func onEstablishConnection() {
        switch state {
        case var .connecting(connState), var .reconnecting(connState):
            // Reset connection attempt once successfully connected.
            connState.connectionAttemptCount = 0
            state = .connected(connState)

        case .initial, .connected, .disconnecting, .disconnected, .error:
            break
        }
    }

    private func onHandleConnectionRecovery() async {
        switch state {
        case var .connecting(connState), var .reconnecting(connState), var .connected(connState):
            guard let targetState = state.targetStateForReconnect else { return }

            // Increment attempt counter
            connState.incrementAttemptCount()

            switch targetState {
            case .connecting:
                state = .connecting(connState)
            case .reconnecting:
                state = .reconnecting(connState)
            }

            // Tunnel monitor should already be paused at this point so don't stop it to avoid a reset of its internal
            // counters.
            try? await reconnectInner(to: .random, shouldStopTunnelMonitor: false)

        case .initial, .disconnected, .disconnecting, .error:
            break
        }
    }
}
