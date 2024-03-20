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
        case let .connecting(connState), let .connected(connState), let .reconnecting(connState),
             let .negotiatingKey(connState):
            connState.relayConstraints

        case let .error(blockedState):
            blockedState.relayConstraints

        case .initial, .disconnecting, .disconnected:
            nil
        }
    }

    public var name: String {
        switch self {
        case .connected:
            "Connected"
        case .connecting:
            "Connecting"
        case .negotiatingKey:
            "Negotiating key"
        case .reconnecting:
            "Reconnecting"
        case .disconnecting:
            "Disconnecting"
        case .disconnected:
            "Disconnected"
        case .initial:
            "Initial"
        case .error:
            "Error"
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
