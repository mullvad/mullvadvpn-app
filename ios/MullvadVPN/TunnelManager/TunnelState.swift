//
//  TunnelState.swift
//  TunnelState
//
//  Created by pronebird on 11/08/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import PacketTunnelCore
@preconcurrency import WireGuardKitTypes

/// Describes the tunnel status.
///
/// The `state` property is reflected in the main view of the app, and typically shows
/// whether the VPN is connected, connecting, or disconnected.
/// On top of that, a banner might be shown in cases `state` is either `waitingForConnectivity` or `error`
///
/// The `observedState` contains metadata about the PacketTunnel, and can be used to infer various details such as
/// - A reason why the app would enter the blocked state
/// - Whether networking is available from within the `PacketTunnel` process
/// - How many times a reconnection was attempted
/// - Which protocol layer is used by the `PacketTunnel` (TCP, UDP etc...)
///
/// And so on, this is a non-exhaustive list.
struct TunnelStatus: Equatable, CustomStringConvertible, Sendable {
    /// Reflects the `PacketTunnel`'s internal state.
    var observedState: ObservedState = .disconnected

    /// Internal state used by the UI Process to manage transitions and UI updates.
    /// Directly affects the UI, what user actions are available.
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
enum TunnelState: Equatable, CustomStringConvertible, Sendable {
    enum WaitingForConnectionReason {
        /// Tunnel connection is down.
        case noConnection
        /// Network is down.
        case noNetwork
    }

    /// Pending reconnect after disconnect.
    case pendingReconnect

    /// Connecting the tunnel.
    case connecting(SelectedRelays?, isPostQuantum: Bool, isDaita: Bool)

    /// Negotiating an ephemeral peer either for post-quantum resistance or Daita
    case negotiatingEphemeralPeer(SelectedRelays, PrivateKey, isPostQuantum: Bool, isDaita: Bool)

    /// Connected the tunnel
    case connected(SelectedRelays, isPostQuantum: Bool, isDaita: Bool)

    /// Disconnecting the tunnel
    case disconnecting(ActionAfterDisconnect)

    /// Disconnected the tunnel
    case disconnected

    /// Reconnecting the tunnel.
    /// Transition to this state happens when:
    /// 1. Asking the running tunnel to reconnect to new relays via IPC.
    /// 2. Tunnel attempts to reconnect to new relays as the current relays appear to be
    ///    dysfunctional.
    case reconnecting(SelectedRelays, isPostQuantum: Bool, isDaita: Bool)

    /// Waiting for connectivity to come back up.
    case waitingForConnectivity(WaitingForConnectionReason)

    /// Error state.
    case error(BlockedStateReason)

    var description: String {
        switch self {
        case .pendingReconnect:
            "pending reconnect after disconnect"
        case let .connecting(tunnelRelays, isPostQuantum, isDaita):
            if let tunnelRelays {
                """
                connecting \(isPostQuantum ? "(PQ)" : ""), \
                daita: \(isDaita), \
                to \(tunnelRelays.exit.hostname)\
                \(tunnelRelays.entry.flatMap { " via \($0.hostname)" } ?? "")
                """
            } else {
                "connecting \(isPostQuantum ? "(PQ)" : ""), fetching relay"
            }
        case let .connected(tunnelRelays, isPostQuantum, isDaita):
            """
            connected \(isPostQuantum ? "(PQ) " : ""), \
            daita: \(isDaita), \
            to \(tunnelRelays.exit.hostname)\
            \(tunnelRelays.entry.flatMap { " via \($0.hostname)" } ?? "")
            """
        case let .disconnecting(actionAfterDisconnect):
            "disconnecting and then \(actionAfterDisconnect)"
        case .disconnected:
            "disconnected"
        case let .reconnecting(tunnelRelays, isPostQuantum, isDaita):
            """
            reconnecting \(isPostQuantum ? "(PQ) " : ""), \
            daita: \(isDaita), \
            to \(tunnelRelays.exit.hostname)\
            \(tunnelRelays.entry.flatMap { " via \($0.hostname)" } ?? "")
            """
        case .waitingForConnectivity:
            "waiting for connectivity"
        case let .error(blockedStateReason):
            "error state: \(blockedStateReason)"
        case let .negotiatingEphemeralPeer(tunnelRelays, _, isPostQuantum, isDaita):
            """
            negotiating key with exit relay: \(tunnelRelays.exit.hostname)\
            \(tunnelRelays.entry.flatMap { " via \($0.hostname)" } ?? ""), \
            isPostQuantum: \(isPostQuantum), isDaita: \(isDaita)
            """
        }
    }

    var isSecured: Bool {
        switch self {
        case .reconnecting, .connecting, .connected, .waitingForConnectivity(.noConnection), .error(.accountExpired),
            .error(.deviceRevoked), .negotiatingEphemeralPeer:
            true
        case .pendingReconnect, .disconnecting, .disconnected, .waitingForConnectivity(.noNetwork), .error:
            false
        }
    }

    var isBlockingInternet: Bool {
        switch self {
        case .connected, .disconnected:
            false
        default:
            true
        }
    }

    var relays: SelectedRelays? {
        switch self {
        case let .connected(relays, _, _),
            let .reconnecting(relays, _, _),
            let .negotiatingEphemeralPeer(relays, _, _, _):
            relays
        case let .connecting(relays, _, _):
            relays
        case .disconnecting, .disconnected, .waitingForConnectivity, .pendingReconnect, .error:
            nil
        }
    }

    // the two accessors below return a Bool?, to differentiate known
    // truth values from undefined/meaningless values, which the caller
    // may want to interpret differently
    var isPostQuantum: Bool? {
        switch self {
        case let .connecting(_, isPostQuantum: isPostQuantum, isDaita: _),
            let .connected(_, isPostQuantum: isPostQuantum, isDaita: _),
            let .reconnecting(_, isPostQuantum: isPostQuantum, isDaita: _):
            isPostQuantum
        default:
            nil
        }
    }

    var isDaita: Bool? {
        switch self {
        case let .connecting(_, isPostQuantum: _, isDaita: isDaita),
            let .connected(_, isPostQuantum: _, isDaita: isDaita),
            let .reconnecting(_, isPostQuantum: _, isDaita: isDaita):
            isDaita
        default:
            nil
        }
    }

    var isMultihop: Bool {
        relays?.entry != nil
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
