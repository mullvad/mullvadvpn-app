//
//  State+.swift
//  PacketTunnelCore
//
//  Created by pronebird on 08/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension State {
    var priorState: StatePriorToBlockedState? {
        switch self {
        case .initial:
            return .initial
        case .connecting:
            return .connecting
        case .connected:
            return .connected
        case .reconnecting:
            return .reconnecting
        case .disconnecting, .disconnected, .error:
            return nil
        }
    }

    /// Calls `body` with `ConnectionState` if the state contains it. Otherwise returns `State` as is.
    func mapConnectionState(_ body: (inout ConnectionState) -> Void) -> State {
        switch self {
        case var .connected(connState):
            body(&connState)
            return .connected(connState)

        case var .connecting(connState):
            body(&connState)
            return .connecting(connState)

        case var .reconnecting(connState):
            body(&connState)
            return .reconnecting(connState)

        case var .disconnecting(connState):
            body(&connState)
            return .disconnecting(connState)

        case .disconnected, .initial, .error:
            return self
        }
    }

    /// Returns the target state to which the actor state should transition when requested to reconnect.
    /// It returns `nil` when reconnection is not supported such as when already `.disconnecting` or `.disconnected` states.
    var targetStateForReconnect: TargetStateForReconnect? {
        switch self {
        case .initial:
            return .connecting

        case var .connecting(connState):
            return .connecting

        case .connected, .reconnecting:
            return .reconnecting

        case let .error(blockedState):
            switch blockedState.priorState {
            case .initial, .connecting:
                return .connecting
            case .connected, .reconnecting:
                return .reconnecting
            }

        case .disconnecting, .disconnected:
            return nil
        }
    }
}
