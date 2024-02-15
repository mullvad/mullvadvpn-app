//
//  Actor+ErrorState.swift
//  PacketTunnelCore
//
//  Created by pronebird on 26/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network
import WireGuardKitTypes

extension PacketTunnelActor {
    /**
     Transition actor to error state.

     Evaluates the error and maps it to `BlockedStateReason` before switching actor to `.error` state.

     - Important: this method will suspend and must only be invoked as a part of channel consumer to guarantee transactional execution.

     - Parameter error: an error that occurred while starting the tunnel.
     */
    func setErrorStateInternal(with error: Error) async {
        let reason = blockedStateErrorMapper.mapError(error)

        await setErrorStateInternal(with: reason)
    }

    /**
     Transition actor to error state.

     Normally actor enters error state on its own, due to unrecoverable errors. However error state can also be induced externally for example in response to
     device check indicating certain issues that actor is not able to detect on its own such as invalid account or device being revoked on backend.

     - Important: this method will suspend and must only be invoked as a part of channel consumer to guarantee transactional execution.

     - Parameter reason: reason why the actor is entering error state.
     */
    func setErrorStateInternal(with reason: BlockedStateReason) async {
        // Tunnel monitor shouldn't run when in error state.
        tunnelMonitor.stop()

        if let blockedState = makeBlockedState(reason: reason) {
            state = .error(blockedState)
            await configureAdapterForErrorState()
        }
    }

    /**
     Derive `BlockedState` from current `state` updating it with the given block reason.

     - Parameter reason: block reason
     - Returns: New blocked state that should be assigned to error state, otherwise `nil` when actor is past or at `disconnecting` phase or
                when actor is already in the error state and no changes need to be made.
     */
    private func makeBlockedState(reason: BlockedStateReason) -> BlockedState? {
        switch state {
        case .initial:
            return BlockedState(
                reason: reason,
                relayConstraints: nil,
                currentKey: nil,
                keyPolicy: .useCurrent,
                networkReachability: defaultPathObserver.defaultPath?.networkReachability ?? .undetermined,
                recoveryTask: startRecoveryTaskIfNeeded(reason: reason),
                priorState: .initial
            )

        case let .connected(connState):
            return mapConnectionState(connState, reason: reason, priorState: .connected)

        case let .connecting(connState):
            return mapConnectionState(connState, reason: reason, priorState: .connecting)

        case let .reconnecting(connState):
            return mapConnectionState(connState, reason: reason, priorState: .reconnecting)

        case var .error(blockedState):
            if blockedState.reason != reason {
                blockedState.reason = reason
                return blockedState
            } else {
                return nil
            }

        case .disconnecting, .disconnected:
            return nil
        }
    }

    /**
     Map connection state to blocked state.
     */
    private func mapConnectionState(
        _ connState: ConnectionState,
        reason: BlockedStateReason,
        priorState: StatePriorToBlockedState
    ) -> BlockedState {
        BlockedState(
            reason: reason,
            relayConstraints: connState.relayConstraints,
            currentKey: connState.currentKey,
            keyPolicy: connState.keyPolicy,
            networkReachability: connState.networkReachability,
            priorState: priorState
        )
    }

    /**
     Configure tunnel with empty WireGuard configuration that consumes all traffic on device emulating a firewall blocking all traffic.
     */
    private func configureAdapterForErrorState() async {
        do {
            let configurationBuilder = ConfigurationBuilder(
                privateKey: PrivateKey(),
                interfaceAddresses: [],
                allowedIPs: []
            )
            var config = try configurationBuilder.makeConfiguration()
            config.dns = [IPv4Address.loopback]
            config.interfaceAddresses = [IPAddressRange(from: "10.64.0.1/8")!]
            config.peer = TunnelPeer(
                endpoint: .ipv4(IPv4Endpoint(string: "127.0.0.1:9090")!),
                publicKey: PrivateKey().publicKey
            )
            try? await tunnelAdapter.stop()
            try await tunnelAdapter.start(configuration: config)
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

        // Use detached task to prevent inheriting current context.
        let task = Task.detached { [weak self] in
            while !Task.isCancelled {
                guard let self else { return }

                try await Task.sleepUsingContinuousClock(for: timings.bootRecoveryPeriodicity)

                // Schedule task to reconnect.
                commandChannel.send(.reconnect(.random))
            }
        }

        return AutoCancellingTask(task)
    }
}
