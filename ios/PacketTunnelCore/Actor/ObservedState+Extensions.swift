//
//  ObservedState+Extensions.swift
//  PacketTunnelCore
//
//  Created by pronebird on 16/10/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension ObservedState {
    public var relayConstraints: RelayConstraints? {
        switch self {
        case let .connecting(connState), let .connected(connState), let .reconnecting(connState):
            return connState.relayConstraints

        case let .error(blockedState):
            return blockedState.relayConstraints

        case .initial, .disconnecting, .disconnected:
            return nil
        }
    }

    public var name: String {
        switch self {
        case .connected:
            return "Connected"
        case .connecting:
            return "Connecting"
        case .reconnecting:
            return "Reconnecting"
        case .disconnecting:
            return "Disconnecting"
        case .disconnected:
            return "Disconnected"
        case .initial:
            return "Initial"
        case .error:
            return "Error"
        }
    }

    public var connectionState: ObservedConnectionState? {
        switch self {
        case
            let .connecting(connectionState),
            let .reconnecting(connectionState),
            let .connected(connectionState),
            let .disconnecting(connectionState):
            connectionState
        default:
            nil
        }
    }

    public var blockedState: ObservedBlockedState? {
        switch self {
        case let .error(blockedState):
            blockedState
        default:
            nil
        }
    }
}
