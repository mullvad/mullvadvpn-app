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

public actor PacketTunnelActor {
    @Published private(set) public var state: State = .initial {
        didSet {
            logger.debug("\(state.logFormat())")
        }
    }

    private let logger = Logger(label: "PacketTunnelActor")
    private let taskQueue = TaskQueue()

    private let tunnelAdapter: TunnelAdapterProtocol
    private let tunnelMonitor: TunnelMonitorProtocol
    private let defaultPathObserver: DefaultPathObserverProtocol
    private let blockedStateErrorMapper: BlockedStateErrorMapperProtocol
    private let relaySelector: RelaySelectorProtocol
    private let settingsReader: SettingsReaderProtocol

    public init(
        tunnelAdapter: TunnelAdapterProtocol,
        tunnelMonitor: TunnelMonitorProtocol,
        defaultPathObserver: DefaultPathObserverProtocol,
        blockedStateErrorMapper: BlockedStateErrorMapperProtocol,
        relaySelector: RelaySelectorProtocol,
        settingsReader: SettingsReaderProtocol
    ) {
        self.tunnelAdapter = tunnelAdapter
        self.tunnelMonitor = tunnelMonitor
        self.defaultPathObserver = defaultPathObserver
        self.blockedStateErrorMapper = blockedStateErrorMapper
        self.relaySelector = relaySelector
        self.settingsReader = settingsReader
    }

    public func start(options: StartOptions) async throws {
        try await taskQueue.add(kind: .start) { [self] in
            guard case .initial = state else { return }

            logger.debug("\(options.logFormat())")

            // Start observing default network path to determine network reachability.
            defaultPathObserver.start { [weak self] networkPath in
                guard let self else { return }
                Task { await self.onDefaultPathChange(networkPath) }
            }

            // Assign a closure receiving tunnel monitor events.
            tunnelMonitor.onEvent = { [weak self] event in
                guard let self else { return }
                Task { await self.handleMonitorEvent(event) }
            }

            do {
                try await tryStart(nextRelay: options.selectorResult.map { .preSelected($0) } ?? .random)
            } catch {
                try Task.checkCancellation()

                logger.error(error: error, message: "Failed to start the tunnel.")

                await setErrorState(with: error)
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

    public func stop() async {
        await taskQueue.add(kind: .stop) { [self] in
            switch state {
            case let .connected(connState), let .connecting(connState), let .reconnecting(connState):
                state = .disconnecting(connState)
                tunnelMonitor.stop()

                // Fallthrough to stop adapter and shift to `.disconnected` state.
                fallthrough

            case .error:
                defaultPathObserver.stop()

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

    public func reconnect(to nextRelay: NextRelay) async throws {
        try await reconnect(to: nextRelay, shouldStopTunnelMonitor: true)
    }

    // MARK: - Private key rotation notifications

    /// Notify actor that the private key rotation took place.
    /// When that happens the actor changes key policy to `.usePrior` in order
    public func notifyKeyRotated() async {
        await taskQueue.add(kind: .keyRotated) { [self] in
            func mutateConnectionState(_ connState: inout ConnectionState) -> Bool {
                switch connState.keyPolicy {
                case .useCurrent:
                    connState.keyPolicy = .usePrior(connState.currentKey, AutoCancellingTask(startSwitchKeyTask()))
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
                        blockedState.keyPolicy = .usePrior(currentKey, AutoCancellingTask(startSwitchKeyTask()))
                        state = .error(blockedState)
                    }

                case .usePrior:
                    break
                }

            case .initial, .disconnected, .disconnecting:
                break
            }
        }
    }

    /// Start a task that sleeps for 120 seconds before switching key policy to `.useCurrent` and reconnecting the tunnel with the new key.
    private func startSwitchKeyTask() -> AnyTask {
        return Task {
            // Wait for key to propagate across relays.
            try await Task.sleep(seconds: 120)

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
                    break
                case .usePrior:
                    blockedState.keyPolicy = .useCurrent
                    state = .error(blockedState)
                }

            case .disconnected, .disconnecting, .initial:
                break
            }

            // This will schedule a normal call to reconnect that will be enqueued on the task queue.
            try await reconnect(to: .random)
        }
    }

    // MARK: - Sleep cycle notifications

    public nonisolated func onWake() {
        tunnelMonitor.onWake()
    }

    public func onSleep() {
        tunnelMonitor.onSleep()
    }

    // MARK: - Network Reachability

    func onDefaultPathChange(_ networkPath: NetworkPath) async {
        await taskQueue.add(kind: .networkReachability) { [self] in
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

    private func tryStart(nextRelay: NextRelay = .random) async throws {
        let settings: Settings = try settingsReader.read()
        guard let storedDeviceData = settings.deviceState.deviceData else { throw BlockedStateError.deviceRevoked }

        func makeConnectionState() throws -> ConnectionState? {
            let relayConstraints = settings.tunnelSettings.relayConstraints
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
                    currentKey: storedDeviceData.wgKeyData.privateKey,
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
                connState.currentKey = storedDeviceData.wgKeyData.privateKey

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
                    currentKey: storedDeviceData.wgKeyData.privateKey,
                    keyPolicy: blockedState.keyPolicy,
                    networkReachability: .undetermined,
                    connectionAttemptCount: 0
                )

            case .disconnecting, .disconnected:
                return nil
            }
        }

        guard let connectionState = try makeConnectionState(),
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
            interfaceAddresses: [storedDeviceData.ipv4Address, storedDeviceData.ipv6Address],
            dns: settings.tunnelSettings.dnsSettings,
            endpoint: endpoint
        )
        try await tunnelAdapter.start(configuration: configurationBuilder.makeConfiguration())

        // Resume tunnel monitoring and use IPv4 gateway as a probe address.
        tunnelMonitor.start(probeAddress: endpoint.ipv4Gateway)
    }

    private func reconnect(to nextRelay: NextRelay, shouldStopTunnelMonitor: Bool) async throws {
        try await taskQueue.add(kind: .reconnect) { [self] in
            try Task.checkCancellation()

            try await reconnectInner(to: nextRelay, shouldStopTunnelMonitor: shouldStopTunnelMonitor)
        }
    }

    private func reconnectInner(to nextRelay: NextRelay, shouldStopTunnelMonitor: Bool) async throws {
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

            await setErrorState(with: error)
        }
    }

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

    private func setErrorState(with error: Error) async {
        switch state {
        case let .connected(connState), let .connecting(connState), let .reconnecting(connState):
            let blockedState = BlockedState(
                reason: blockedStateErrorMapper.mapError(error),
                relayConstraints: connState.relayConstraints,
                currentKey: nil,
                keyPolicy: connState.keyPolicy,
                networkReachability: connState.networkReachability,
                priorState: state.priorState!
            )
            state = .error(blockedState)
            await configureAdapterForErrorState()

        case .initial:
            var blockedState = BlockedState(
                reason: blockedStateErrorMapper.mapError(error),
                relayConstraints: nil,
                currentKey: nil,
                keyPolicy: .useCurrent,
                networkReachability: defaultPathObserver.defaultPath?.networkReachability ?? .undetermined,
                recoveryTask: nil,
                priorState: state.priorState!
            )

            // Create a recovery task if the tunnel can recover from error state automatically
            if blockedState.reason.shouldRestartAutomatically {
                blockedState.recoveryTask = AutoCancellingTask(startRecoveryTask())
            }

            state = .error(blockedState)
            await configureAdapterForErrorState()

        case var .error(blockedState):
            blockedState.reason = blockedStateErrorMapper.mapError(error)
            state = .error(blockedState)

        case .disconnecting, .disconnected:
            break
        }
    }

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

    private func startRecoveryTask() -> AnyTask {
        return Task { [weak self] in
            let repeating: DispatchTimeInterval = .seconds(10)
            let timerStream = DispatchSource.scheduledTimer(on: .now() + repeating, repeating: repeating)

            for await _ in timerStream {
                await self?.onRecoveryTimer()
            }
        }
    }

    private func onRecoveryTimer() async {
        try? await taskQueue.add(kind: .start) { [self] in
            guard case .error = self.state else { return }

            do {
                try await tryStart()
            } catch {
                try Task.checkCancellation()

                logger.error(error: error, message: "Failed to start the tunnel from recovery timer.")

                await setErrorState(with: error)
            }
        }
    }

    // MARK: - Private: Connection monitoring

    private func onEstablishConnection() async {
        switch state {
        case let .connecting(connState), let .reconnecting(connState):
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

            // Tunnel monitor should already be paused at this point so don't stop it to avoid reseting its internal
            // counters.
            try? await reconnect(to: .random, shouldStopTunnelMonitor: false)

        case .initial, .disconnected, .disconnecting, .error:
            break
        }
    }

    private func onNetworkReachibilityChange(_ isNetworkReachable: Bool) async {
        func mutateConnectionState(_ connState: inout ConnectionState) -> Bool {
            let networkReachability: NetworkReachability = isNetworkReachable ? .reachable : .unreachable

            if connState.networkReachability != networkReachability {
                connState.networkReachability = networkReachability
                return true
            }

            return false
        }

        switch state {
        case var .connected(connState):
            if mutateConnectionState(&connState) {
                state = .connected(connState)
            }

        case var .connecting(connState):
            if mutateConnectionState(&connState) {
                state = .connecting(connState)
            }

        case var .reconnecting(connState):
            if mutateConnectionState(&connState) {
                state = .reconnecting(connState)
            }

        case var .disconnecting(connState):
            if mutateConnectionState(&connState) {
                state = .disconnecting(connState)
            }

        case .disconnected, .initial, .error:
            break
        }
    }

    private func handleMonitorEvent(_ event: TunnelMonitorEvent) async {
        switch event {
        case .connectionEstablished:
            await onEstablishConnection()

        case .connectionLost:
            await onHandleConnectionRecovery()

        case .networkReachabilityChanged:
            // TODO: remove once networkReachabilityChanged later
            // We track network reachability separately because tunnel monitor tends to stop reachability
            // observation when it's stopped or paused. This is a problem for error state.
            break
        }
    }
}
