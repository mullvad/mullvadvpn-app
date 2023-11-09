//
//  TunnelState.swift
//  TunnelState
//
//  Created by pronebird on 11/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import PacketTunnelCore

/// A struct describing the tunnel status.
struct TunnelStatus: Equatable, CustomStringConvertible {
    /// Tunnel status returned by tunnel process.
    var observedState: ObservedState = .disconnected

    /// Tunnel state.
    var state: TunnelState = .disconnected

    var description: String {
        var s = "\(state), network "

        if let connectionState = observedState.connectionState {
            if connectionState.isNetworkReachable {
                s += "reachable"
            } else {
                s += "unreachable"
            }
        } else {
            s += "reachability unknown"
        }

        return s
    }
}

/// An enum that describes the tunnel state.
enum TunnelState: Equatable, CustomStringConvertible {
    enum WaitingForConnectionReason {
        /// Tunnel connection is down.
        case noConnection
        /// Network is down.
        case noNetwork
    }

    /// Pending reconnect after disconnect.
    case pendingReconnect

    /// Connecting the tunnel.
    case connecting(SelectedRelay?)

    /// Connected the tunnel
    case connected(SelectedRelay)

    /// Disconnecting the tunnel
    case disconnecting(ActionAfterDisconnect)

    /// Disconnected the tunnel
    case disconnected

    /// Reconnecting the tunnel.
    /// Transition to this state happens when:
    /// 1. Asking the running tunnel to reconnect to new relay via IPC.
    /// 2. Tunnel attempts to reconnect to new relay as the current relay appears to be
    ///    dysfunctional.
    case reconnecting(SelectedRelay)

    /// Waiting for connectivity to come back up.
    case waitingForConnectivity(WaitingForConnectionReason)

    /// Error state.
    case error(BlockedStateReason)

    var description: String {
        switch self {
        case .pendingReconnect:
            return "pending reconnect after disconnect"
        case let .connecting(tunnelRelay):
            if let tunnelRelay {
                return "connecting to \(tunnelRelay.hostname)"
            } else {
                return "connecting, fetching relay"
            }
        case let .connected(tunnelRelay):
            return "connected to \(tunnelRelay.hostname)"
        case let .disconnecting(actionAfterDisconnect):
            return "disconnecting and then \(actionAfterDisconnect)"
        case .disconnected:
            return "disconnected"
        case let .reconnecting(tunnelRelay):
            return "reconnecting to \(tunnelRelay.hostname)"
        case .waitingForConnectivity:
            return "waiting for connectivity"
        case let .error(blockedStateReason):
            return "error state: \(blockedStateReason)"
        }
    }

    var isSecured: Bool {
        switch self {
        case .reconnecting, .connecting, .connected, .waitingForConnectivity(.noConnection), .error(.accountExpired):
            return true
        case .pendingReconnect, .disconnecting, .disconnected, .waitingForConnectivity(.noNetwork), .error:
            return false
        }
    }

    var relay: SelectedRelay? {
        switch self {
        case let .connected(relay), let .reconnecting(relay):
            return relay
        case let .connecting(relay):
            return relay
        case .disconnecting, .disconnected, .waitingForConnectivity, .pendingReconnect, .error:
            return nil
        }
    }
}

/// A enum that describes the action to perform after disconnect.
enum ActionAfterDisconnect: CustomStringConvertible {
    /// Do nothing after disconnecting.
    case nothing

    /// Reconnect after disconnecting.
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
