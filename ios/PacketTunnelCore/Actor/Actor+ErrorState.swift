//
//  Actor+ErrorState.swift
//  PacketTunnelCore
//
//  Created by pronebird on 26/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import class WireGuardKitTypes.PrivateKey

extension PacketTunnelActor {
    /**
     Switch actor into error state.

     Normally actor enters error state on its own, due to unrecoverable errors. However error state can also be induced externally for example in response to
     device check indicating certain issues that actor is not able to detect on its own such as invalid account or device being revoked on backend.

     - Important: Internal implementation must not call this method from operation executing on `TaskQueue`.

     - Parameter reason: block reason
     */
    public func setErrorState(with reason: BlockedStateReason) async {
        try? await taskQueue.add(kind: .blockedState) { [self] in
            try Task.checkCancellation()
            await setErrorStateInner(with: reason)
        }
    }

    /**
     Transition actor to error state.

     Evaluates the error and maps it to `BlockedStateReason` before switching actor to `.error` state.

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue`.

     - Parameter error: an error that occurred while starting the tunnel.
     */
    func setErrorStateInner(with error: Error) async {
        let reason = blockedStateErrorMapper.mapError(error)

        await setErrorStateInner(with: reason)
    }

    /**
     Transition actor to error state.

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue`.

     - Parameter reason: reason why the actor is entering error state.
     */
    func setErrorStateInner(with reason: BlockedStateReason) async {
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
                priorState: state.priorState!
            )

        case let .connected(connState), let .connecting(connState), let .reconnecting(connState):
            return BlockedState(
                reason: reason,
                relayConstraints: connState.relayConstraints,
                currentKey: nil,
                keyPolicy: connState.keyPolicy,
                networkReachability: connState.networkReachability,
                priorState: state.priorState!
            )

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
     Configure tunnel with empty WireGuard configuration that consumes all traffic on device and emitates the blocked state akin to the one we have on desktop
     which however utilizes firewall to achieve this.

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue`.
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
}
