//
//  TunnelState.swift
//  TunnelState
//
//  Created by pronebird on 11/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A enum that describes the tunnel state
enum TunnelState: Equatable, CustomStringConvertible {
    /// Pending reconnect after disconnect.
    case pendingReconnect

    /// Connecting the tunnel. Contains the pending action carried over from disconnected state.
    case connecting(TunnelConnectionInfo?)

    /// Connected the tunnel
    case connected(TunnelConnectionInfo)

    /// Disconnecting the tunnel
    case disconnecting(ActionAfterDisconnect)

    /// Disconnected the tunnel
    case disconnected

    /// Reconnecting the tunnel. Normally this state appears in response to changing the
    /// relay constraints and asking the running tunnel to reload the configuration.
    case reconnecting(TunnelConnectionInfo)

    var description: String {
        switch self {
        case .pendingReconnect:
            return "pending reconnect after disconnect"
        case .connecting(let connectionInfo):
            if let connectionInfo = connectionInfo {
                return "connecting to \(connectionInfo.hostname)"
            } else {
                return "connecting, fetching relay"
            }
        case .connected(let connectionInfo):
            return "connected to \(connectionInfo.hostname)"
        case .disconnecting(let actionAfterDisconnect):
            return "disconnecting and then \(actionAfterDisconnect)"
        case .disconnected:
            return "disconnected"
        case .reconnecting(let connectionInfo):
            return "reconnecting to \(connectionInfo.hostname)"
        }
    }
}

/// A enum that describes the action to perform after disconnect
enum ActionAfterDisconnect {
    /// Do nothing after disconnecting
    case nothing

    /// Reconnect after disconnecting
    case reconnect

    var description: String {
        switch self {
        case .nothing:
            return "do nothing"
        case .reconnect:
            return "reconnect"
        }
    }
}
