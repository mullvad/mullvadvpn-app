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
    @Published private(set) public var state: State = .initial

    private let logger = Logger(label: "PacketTunnelActor")
    private let taskQueue = TaskQueue()

    private let tunnelAdapter: TunnelAdapterProtocol
    private let tunnelMonitor: TunnelMonitorProtocol
    private let relaySelector: RelaySelectorProtocol
    private let settingsReader: SettingsReaderProtocol

    // MARK: - Tunnel control

    public init(
        tunnelAdapter: TunnelAdapterProtocol,
        tunnelMonitor: TunnelMonitorProtocol,
        relaySelector: RelaySelectorProtocol,
        settingsReader: SettingsReaderProtocol
    ) {
        self.tunnelAdapter = tunnelAdapter
        self.tunnelMonitor = tunnelMonitor
        self.relaySelector = relaySelector
        self.settingsReader = settingsReader

        tunnelMonitor.onEvent = { [weak self] event in
            guard let self else { return }

            Task {
                await self.handleMonitorEvent(event)
            }
        }
    }

    public func start(options: StartOptions) async throws {
        try await taskQueue.add(kind: .start) { [self] in
            guard case .initial = state else { return }

            logger.debug("\(options.logFormat())")

            do {
                try await tryStart(selectorResult: options.selectorResult)
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

    public func reconnect(to selectorResult: RelaySelectorResult?) async throws {
        try await reconnect(to: selectorResult, shouldStopTunnelMonitor: true)
    }

    // MARK: - Sleep cycle notifications

    public nonisolated func onWake() {
        tunnelMonitor.onWake()
    }

    public func onSleep() {
        tunnelMonitor.onSleep()
    }

    // MARK: - Private: Tunnel management

    private func tryStart(selectorResult: RelaySelectorResult? = nil) async throws {
        let settings: Settings = try settingsReader.read()
        guard let storedDeviceData = settings.deviceState.deviceData else { throw DeviceRevokedError() }

        func selectRelay() throws -> RelaySelectorResult {
            return try selectorResult ?? relaySelector.selectRelay(
                with: settings.tunnelSettings.relayConstraints,
                connectionAttemptFailureCount: 0
            )
        }

        func makeConnectionState() throws -> ConnectionState? {
            switch state {
            case .initial:
                return ConnectionState(
                    selectedRelay: try selectRelay(),
                    currentKey: storedDeviceData.wgKeyData.privateKey,
                    keyPolicy: .useCurrent,
                    networkReachability: .undetermined,
                    connectionAttemptCount: 0
                )

            case var .connecting(connState), var .connected(connState), var .reconnecting(connState):
                connState.selectedRelay = try selectRelay()
                connState.currentKey = storedDeviceData.wgKeyData.privateKey

                return connState

            case let .error(blockedState):
                return ConnectionState(
                    selectedRelay: try selectRelay(),
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

    private func reconnect(to selectorResult: RelaySelectorResult?, shouldStopTunnelMonitor: Bool) async throws {
        try await taskQueue.add(kind: .reconnect) { [self] in
            try Task.checkCancellation()

            do {
                switch state {
                case .connecting, .connected, .reconnecting, .error:
                    if shouldStopTunnelMonitor {
                        tunnelMonitor.stop()
                    }
                    try await tryStart(selectorResult: selectorResult)

                case .disconnected, .disconnecting, .initial:
                    break
                }
            } catch {
                try Task.checkCancellation()

                logger.error(error: error, message: "Failed to reconnect the tunnel.")

                await setErrorState(with: error)
            }
        }
    }

    // MARK: - Private: Error state

    private func setErrorState(with error: Error) async {
        switch state {
        case let .connected(connState), let .connecting(connState), let .reconnecting(connState):
            let blockedState = BlockedState(
                error: error,
                keyPolicy: connState.keyPolicy,
                priorState: state.priorState!
            )
            state = .error(blockedState)
            await configureAdapterForErrorState()

        case .initial:
            // Start a recovery task that will attempt to restart the tunnel periodically.
            //
            // Often times this has to be done when the tunnel is started on boot and cannot access Keychain or
            // filesystem (FS) which are locked until the phone is unlocked first.
            //
            // TODO: only start the recovery task if the error is filesystem/Keychain permission related.
            let recoveryTask = startRecoveryTask()

            let blockedState = BlockedState(
                error: error,
                keyPolicy: .useCurrent,
                recoveryTask: AutoCancellingTask(recoveryTask),
                priorState: state.priorState!
            )
            state = .error(blockedState)
            await configureAdapterForErrorState()

        case var .error(blockedState):
            blockedState.error = error
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
            logger.debug("Connection established.")
            state = .connected(connState)

        case .initial, .connected, .disconnecting, .disconnected, .error:
            break
        }
    }

    private func onHandleConnectionRecovery() async {
        switch state {
        case var .connecting(connState), var .reconnecting(connState), var .connected(connState):
            // Increment attempt counter
            connState.incrementAttemptCount()

            // Remain in connecting state if we haven't been able to connect in the first place.
            if case .connecting = state {
                state = .connecting(connState)
            } else {
                state = .reconnecting(connState)
            }

            if connState.connectionAttemptCount.isMultiple(of: 2) {
                // TODO: start device check
            }

            logger.debug("Recover connection. Picking next relay...")

            // Tunnel monitor should already be paused at this point.
            try? await reconnect(to: nil, shouldStopTunnelMonitor: false)

        case .initial, .disconnected, .disconnecting, .error:
            break
        }
    }

    private func onNetworkReachibilityChange(_ isNetworkReachable: Bool) async {
        state = state.mapConnectionState { connState in
            connState.networkReachability = isNetworkReachable ? .reachable : .unreachable
        }
    }

    private func handleMonitorEvent(_ event: TunnelMonitorEvent) async {
        switch event {
        case .connectionEstablished:
            await onEstablishConnection()

        case .connectionLost:
            await onHandleConnectionRecovery()

        case let .networkReachabilityChanged(isNetworkReachable):
            await onNetworkReachibilityChange(isNetworkReachable)
        }
    }
}
