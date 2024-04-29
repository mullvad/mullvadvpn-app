//
//  PacketTunnelActorReducer.swift
//  PacketTunnelCore
//
//  Created by Andrew Bulhak on 2024-04-26.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import WireGuardKitTypes

extension WireGuardKey where Self: Equatable {}

extension PacketTunnelActor {
    ///  A structure encoding an effect; each event will yield zero or more of those, which can then be sequentially executed.
    enum Effect: Equatable {
        case startDefaultPathObserver
        case stopDefaultPathObserver
        case startTunnelMonitor
        case stopTunnelMonitor
        case updateTunnelMonitorPath(NetworkPath)
        case startConnection(NextRelay)
        case restartConnection(NextRelay, ReconnectReason)
        // trigger a reconnect, which becomes several effects depending on the state
        case reconnect(NextRelay)
        case stopTunnelAdapter
        case configureForErrorState(BlockedStateReason)
        case cacheActiveKey(Date?)
        case postQuantumConnect(PreSharedKey, privateKey: PrivateKey)

        // We cannot synthesise Equatable on Effect because NetworkPath is a protocol which cannot be easily made Equatable, so we need to do this for now.
        static func == (lhs: PacketTunnelActor.Effect, rhs: PacketTunnelActor.Effect) -> Bool {
            return switch (lhs, rhs) {
            case (.startDefaultPathObserver, .startDefaultPathObserver): true
            case (.stopDefaultPathObserver, .stopDefaultPathObserver): true
            case (.startTunnelMonitor, .startTunnelMonitor): true
            case (.stopTunnelMonitor, .stopTunnelMonitor): true
            case let (.updateTunnelMonitorPath(lp), .updateTunnelMonitorPath(rp)): lp.status == rp.status
            case let (.startConnection(nr0), .startConnection(nr1)): nr0 == nr1
            case let (.restartConnection(nr0, rr0), .restartConnection(nr1, rr1)): nr0 == nr1 && rr0 == rr1
            case let (.reconnect(nr0), .reconnect(nr1)): nr0 == nr1
            case (.stopTunnelAdapter, .stopTunnelAdapter): true
            case let (.configureForErrorState(r0), .configureForErrorState(r1)): r0 == r1
            case let (.cacheActiveKey(d0), .cacheActiveKey(d1)): d0 == d1
            case let (.postQuantumConnect(psk0, privateKey: pk0), .postQuantumConnect(psk1, privateKey: pk1)): psk0 ==
                psk1 && pk0 == pk1
            default: false
            }
        }
    }

    static func reducer(_ state: inout State, _ command: Command) -> [Effect] {
        switch command {
        case let .start(options):
            guard case .initial = state else { return [] }
            return [
                .startDefaultPathObserver,
                .startTunnelMonitor,
                .startConnection(options.selectedRelay.map { .preSelected($0) } ?? .random),
            ]
        case .stop:
            switch state {
            case let .connected(connState), let .connecting(connState), let .reconnecting(connState),
                 let .negotiatingPostQuantumKey(connState, _):
                state = .disconnecting(connState)
                return [
                    .stopTunnelMonitor,
                    .stopDefaultPathObserver,
                    .stopTunnelAdapter,
                ]
            case .error:
                state = .disconnected
                return [
                    .stopDefaultPathObserver,
                    .stopTunnelAdapter,
                ]

            case .initial, .disconnected:
                return []

            case .disconnecting:
                assertionFailure("stop(): out of order execution.")
                return []
            }

        case let .reconnect(nextRelay, reason: reason):
            switch state {
            case .disconnected, .disconnecting, .initial, .negotiatingPostQuantumKey:
                // There is no connection monitoring going on when exchanging keys.
                // The procedure starts from scratch for each reconnection attempts.
                return []
            case .connecting, .connected, .reconnecting, .error:
                if reason == .userInitiated {
                    return [.stopTunnelMonitor, .restartConnection(nextRelay, reason)]
                } else {
                    return [.restartConnection(nextRelay, reason)]
                }
            }
        case let .error(reason):
            // the transition from error to blocked state currently has side-effects, so will be handled as an effect for now.
            return [.configureForErrorState(reason)]

        case let .notifyKeyRotated(lastKeyRotation):
            // the cacheActiveKey operation is currently effectful, starting a key-switch task within the mutation of state, so this is entirely done in an effect. Perhaps teasing effects out of state mutation is a future refactoring?

            guard state.keyPolicy == .useCurrent else { return [] }

            return [.cacheActiveKey(lastKeyRotation)]

        case .switchKey:
            let oldKeyPolicy = state.keyPolicy
            state.mutateKeyPolicy { keyPolicy in
                if case .usePrior = keyPolicy {
                    keyPolicy = .useCurrent
                }
            }
            if case .error = state { return [] }
            return state.keyPolicy != oldKeyPolicy ? [.reconnect(.random)] : []

        case let .monitorEvent(event):
            switch event {
            case .connectionEstablished:
                switch state {
                case var .connecting(connState), var .reconnecting(connState):
                    // Reset connection attempt once successfully connected.
                    connState.connectionAttemptCount = 0
                    state = .connected(connState)

                case .initial, .connected, .disconnecting, .disconnected, .error, .negotiatingPostQuantumKey:
                    break
                }
                return []
            case .connectionLost:
                switch state {
                case .connecting, .reconnecting, .connected:
                    return [.restartConnection(.random, .connectionLoss)]
                case .initial, .disconnected, .disconnecting, .error, .negotiatingPostQuantumKey:
                    return []
                }
            }

        case let .networkReachability(defaultPath):
            let newReachability = defaultPath.networkReachability
            state.mutateAssociatedData { $0.networkReachability = newReachability }
            return [.updateTunnelMonitorPath(defaultPath)]

        case let .replaceDevicePrivateKey(key, ephemeralKey: ephemeralKey):
            return [.postQuantumConnect(key, privateKey: ephemeralKey)]
        }
    }

    func runReducer(_ command: Command) -> [Effect] {
        PacketTunnelActor.reducer(&state, command)
    }
}
