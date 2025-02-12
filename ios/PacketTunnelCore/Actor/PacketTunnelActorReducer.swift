//
//  PacketTunnelActorReducer.swift
//  PacketTunnelCore
//
//  Created by Andrew Bulhak on 2024-05-22.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network
import WireGuardKitTypes

extension PacketTunnelActor {
    ///  A structure encoding an effect; each event will yield zero or more of those, which can then be sequentially executed.
    enum Effect: Equatable, Sendable {
        case startDefaultPathObserver
        case stopDefaultPathObserver
        case startTunnelMonitor
        case stopTunnelMonitor
        case updateTunnelMonitorPath(Network.NWPath.Status)
        case startConnection(NextRelays)
        case restartConnection(NextRelays, ActorReconnectReason)

        // trigger a reconnect, which becomes several effects depending on the state
        case reconnect(NextRelays)
        case stopTunnelAdapter
        case configureForErrorState(BlockedStateReason)
        case cacheActiveKey(Date?)
        case reconfigureForEphemeralPeer(EphemeralPeerNegotiationState, OneshotChannel)
        case connectWithEphemeralPeer

        // acknowledge that the disconnection process has concluded, go to .disconnected.
        case setDisconnectedState

        // We cannot synthesise Equatable on Effect because NetworkPath is a protocol which cannot be easily made Equatable, so we need to do this for now.
        static func == (lhs: PacketTunnelActor.Effect, rhs: PacketTunnelActor.Effect) -> Bool {
            return switch (lhs, rhs) {
            case (.startDefaultPathObserver, .startDefaultPathObserver): true
            case (.stopDefaultPathObserver, .stopDefaultPathObserver): true
            case (.startTunnelMonitor, .startTunnelMonitor): true
            case (.stopTunnelMonitor, .stopTunnelMonitor): true
            case let (.updateTunnelMonitorPath(lp), .updateTunnelMonitorPath(rp)): lp == rp
            case let (.startConnection(nr0), .startConnection(nr1)): nr0 == nr1
            case let (.restartConnection(nr0, rr0), .restartConnection(nr1, rr1)): nr0 == nr1 && rr0 == rr1
            case let (.reconnect(nr0), .reconnect(nr1)): nr0 == nr1
            case (.stopTunnelAdapter, .stopTunnelAdapter): true
            case let (.configureForErrorState(r0), .configureForErrorState(r1)): r0 == r1
            case let (.cacheActiveKey(d0), .cacheActiveKey(d1)): d0 == d1
            case let (.reconfigureForEphemeralPeer(eph0, _), .reconfigureForEphemeralPeer(eph1, _)): eph0 == eph1
            case (.connectWithEphemeralPeer, .connectWithEphemeralPeer): true
            case (.setDisconnectedState, .setDisconnectedState): true
            default: false
            }
        }
    }

    struct Reducer: Sendable {
        static func reduce(_ state: inout State, _ event: Event) -> [Effect] {
            switch event {
            case let .start(options):
                guard case .initial = state else { return [] }
                return [
                    .startDefaultPathObserver,
                    .startTunnelMonitor,
                    .startConnection(options.selectedRelays.map { .preSelected($0) } ?? .random),
                ]
            case .stop:
                return subreducerForStop(&state)

            case let .reconnect(nextRelay, reason: reason):
                return subreducerForReconnect(state, reason, nextRelay)

            case let .error(reason):
                // the transition from error to blocked state currently has side-effects, so will be handled as an effect for now.
                return [.configureForErrorState(reason)]

            case let .notifyKeyRotated(lastKeyRotation):
                // the cacheActiveKey operation is currently effectful, starting a key-switch task within the mutation of state, so this is entirely done in an effect. Perhaps teasing effects out of state mutation is a future refactoring?
                guard state.keyPolicy == .useCurrent else { return [] }
                return [.cacheActiveKey(lastKeyRotation)]

            case .switchKey:
                return subreducerForSwitchKey(&state)

            case let .monitorEvent(event):
                return subreducerForTunnelMonitorEvent(event, &state)

            case let .networkReachability(defaultPath):
                let newReachability = defaultPath.networkReachability
                state.mutateAssociatedData { $0.networkReachability = newReachability }
                return [.updateTunnelMonitorPath(defaultPath)]

            case let .ephemeralPeerNegotiationStateChanged(configuration, reconfigurationSemaphore):
                return [.reconfigureForEphemeralPeer(configuration, reconfigurationSemaphore)]

            case .notifyEphemeralPeerNegotiated:
                return [.connectWithEphemeralPeer]
            }
        }

        // Parts of the reducer path broken out for specific incoming events

        fileprivate static func subreducerForStop(_ state: inout State) -> [PacketTunnelActor.Effect] {
            //  a call of the reducer produces one state transition and a sequence of effects. In the app, a stop transitions to .disconnecting, shuts down various processes, and finally transitions to .disconnected. We currently do this by having an effect which acknowledges the completion of disconnection and just sets the state. This is a bit messy, and could possibly do with some rethinking.
            switch state {
            case let .connected(connState), let .connecting(connState), let .reconnecting(connState),
                 let .negotiatingEphemeralPeer(connState, _):
                state = .disconnecting(connState)
                return [
                    .stopTunnelMonitor,
                    .stopDefaultPathObserver,
                    .stopTunnelAdapter,
                    .setDisconnectedState,
                ]
            case .error:
                return [
                    .stopDefaultPathObserver,
                    .stopTunnelAdapter,
                    .setDisconnectedState,
                ]

            case .initial, .disconnected:
                return []

            case .disconnecting:
                assertionFailure("stop(): out of order execution.")
                return []
            }
        }

        fileprivate static func subreducerForReconnect(
            _ state: State,
            _ reason: ActorReconnectReason,
            _ nextRelays: NextRelays
        ) -> [PacketTunnelActor.Effect] {
            switch state {
            case .disconnected, .disconnecting, .initial:
                // There is no connection monitoring going on when exchanging keys.
                // The procedure starts from scratch for each reconnection attempts.
                return []
            case .connecting, .connected, .reconnecting, .error, .negotiatingEphemeralPeer:
                if reason == .userInitiated {
                    return [.stopTunnelMonitor, .restartConnection(nextRelays, reason)]
                } else {
                    return [.restartConnection(nextRelays, reason)]
                }
            }
        }

        fileprivate static func subreducerForSwitchKey(_ state: inout State) -> [PacketTunnelActor.Effect] {
            let oldKeyPolicy = state.keyPolicy
            state.mutateKeyPolicy { keyPolicy in
                if case .usePrior = keyPolicy {
                    keyPolicy = .useCurrent
                }
            }
            if case .error = state { return [] }
            return state.keyPolicy != oldKeyPolicy ? [.reconnect(.random)] : []
        }

        fileprivate static func subreducerForTunnelMonitorEvent(
            _ event: TunnelMonitorEvent,
            _ state: inout State
        ) -> [PacketTunnelActor.Effect] {
            switch event {
            case .connectionEstablished:
                switch state {
                case var .connecting(connState), var .reconnecting(connState):
                    // Reset connection attempt once successfully connected.
                    connState.connectionAttemptCount = 0
                    state = .connected(connState)

                case .initial, .connected, .disconnecting, .disconnected, .error, .negotiatingEphemeralPeer:
                    break
                }
                return []
            case .connectionLost:
                switch state {
                case .connecting, .reconnecting, .connected:
                    return [.restartConnection(.random, .connectionLoss)]
                case .initial, .disconnected, .disconnecting, .error, .negotiatingEphemeralPeer:
                    return []
                }
            }
        }
    }

    func runReducer(_ event: Event) -> [Effect] {
        PacketTunnelActor.Reducer.reduce(&state, event)
    }
}
