//
//  State+.swift
//  PacketTunnel
//
//  Created by pronebird on 12/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore
import MullvadTypes

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
            // TODO: pass errors to UI process
            status.lastErrors = []
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

    func logFormat() -> String {
        switch self {
        case let .connecting(connState), let .connected(connState), let .reconnecting(connState):
            let hostname = connState.selectedRelay.packetTunnelRelay.hostname

            return """
            \(name) to \(hostname) using \(connState.keyPolicy.logFormat()), \
            network is \(connState.networkReachability), \
            connection \(connState.connectionAttemptCount)
            """

        case let .error(blockedState):
            return "\(name): \(blockedState.error.localizedDescription)"

        case .initial, .disconnecting, .disconnected:
            return name
        }
    }

    var name: String {
        switch self {
        case .connected:
            return "connected"
        case .connecting:
            return "connecting"
        case .reconnecting:
            return "reconnecting"
        case .disconnecting:
            return "disconnecting"
        case .disconnected:
            return "disconnected"
        case .initial:
            return "initial"
        case .error:
            return "error"
        }
    }
}

extension KeyPolicy {
    func logFormat() -> String {
        switch self {
        case .useCurrent:
            return "current key"
        case .usePrior:
            return "prior key"
        }
    }
}
