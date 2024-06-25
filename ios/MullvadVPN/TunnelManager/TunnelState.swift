//
//  TunnelState.swift
//  TunnelState
//
//  Created by pronebird on 11/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import PacketTunnelCore
import WireGuardKitTypes

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
    case connecting(SelectedRelays?, isPostQuantum: Bool)

    /// Negotiating a key for post-quantum resistance
    case negotiatingPostQuantumKey(SelectedRelays, PrivateKey)

    /// Connected the tunnel
    case connected(SelectedRelays, isPostQuantum: Bool)

    /// Disconnecting the tunnel
    case disconnecting(ActionAfterDisconnect)

    /// Disconnected the tunnel
    case disconnected

    /// Reconnecting the tunnel.
    /// Transition to this state happens when:
    /// 1. Asking the running tunnel to reconnect to new relays via IPC.
    /// 2. Tunnel attempts to reconnect to new relays as the current relays appear to be
    ///    dysfunctional.
    case reconnecting(SelectedRelays, isPostQuantum: Bool)

    /// Waiting for connectivity to come back up.
    case waitingForConnectivity(WaitingForConnectionReason)

    /// Error state.
    case error(BlockedStateReason)

    var description: String {
        return switch self {
        case .pendingReconnect:
            "pending reconnect after disconnect"
        case let .connecting(tunnelRelays, isPostQuantum):
            if let tunnelRelays {
                """
                connecting \(isPostQuantum ? "(PQ) " : "")\
                to \(tunnelRelays.exit.hostname)\
                \(tunnelRelays.entry.flatMap { " via \($0.hostname)" } ?? "")
                """
            } else {
                "connecting\(isPostQuantum ? " (PQ)" : ""), fetching relay"
            }
        case let .connected(tunnelRelays, isPostQuantum):
            """
            connected \(isPostQuantum ? "(PQ) " : "")\
            to \(tunnelRelays.exit.hostname)\
            \(tunnelRelays.entry.flatMap { " via \($0.hostname)" } ?? "")
            """
        case let .disconnecting(actionAfterDisconnect):
            "disconnecting and then \(actionAfterDisconnect)"
        case .disconnected:
            "disconnected"
        case let .reconnecting(tunnelRelays, isPostQuantum):
            """
            reconnecting \(isPostQuantum ? "(PQ) " : "")\
            to \(tunnelRelays.exit.hostname)\
            \(tunnelRelays.entry.flatMap { " via \($0.hostname)" } ?? "")
            """
        case .waitingForConnectivity:
            "waiting for connectivity"
        case let .error(blockedStateReason):
            "error state: \(blockedStateReason)"
        case let .negotiatingPostQuantumKey(tunnelRelays, _):
            """
            negotiating key with exit relay: \(tunnelRelays.exit.hostname)\
            \(tunnelRelays.entry.flatMap { " via \($0.hostname)" } ?? "")
            """
        }
    }

    var isSecured: Bool {
        switch self {
        case .reconnecting, .connecting, .connected, .waitingForConnectivity(.noConnection), .error(.accountExpired),
             .error(.deviceRevoked), .negotiatingPostQuantumKey:
            true
        case .pendingReconnect, .disconnecting, .disconnected, .waitingForConnectivity(.noNetwork), .error:
            false
        }
    }

    var relays: SelectedRelays? {
        switch self {
        case let .connected(relays, _), let .reconnecting(relays, _), let .negotiatingPostQuantumKey(relays, _):
            relays
        case let .connecting(relays, _):
            relays
        case .disconnecting, .disconnected, .waitingForConnectivity, .pendingReconnect, .error:
            nil
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
            "do nothing"
        case .reconnect:
            "reconnect"
        }
    }
}
