//
//  State+Extensions.swift
//  PacketTunnel
//
//  Created by pronebird on 12/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import PacketTunnelCore

extension State {
    var packetTunnelStatus: PacketTunnelStatus {
        var status = PacketTunnelStatus()

        switch self {
        case let .connecting(connState),
             let .connected(connState),
             let .reconnecting(connState),
             let .disconnecting(connState):
            switch connState.networkReachability {
            case .reachable:
                status.isNetworkReachable = true
            case .unreachable:
                status.isNetworkReachable = false
            case .undetermined:
                // TODO: fix me
                status.isNetworkReachable = true
            }

            status.numberOfFailedAttempts = connState.connectionAttemptCount
            status.tunnelRelay = connState.selectedRelay.packetTunnelRelay

        case .disconnected, .initial:
            break

        case let .error(blockedState):
            status.blockedStateReason = blockedState.reason
        }

        return status
    }

    var relayConstraints: RelayConstraints? {
        switch self {
        case let .connecting(connState), let .connected(connState), let .reconnecting(connState):
            return connState.relayConstraints

        case let .error(blockedState):
            return blockedState.relayConstraints

        case .initial, .disconnecting, .disconnected:
            return nil
        }
    }
}
