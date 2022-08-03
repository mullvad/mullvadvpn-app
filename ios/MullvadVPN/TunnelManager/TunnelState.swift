//
//  TunnelState.swift
//  TunnelState
//
//  Created by pronebird on 11/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A struct describing the tunnel status.
struct TunnelStatus: Equatable, CustomStringConvertible {
    /// Tunnel status returned by the tunnel process.
    var packetTunnelStatus = PacketTunnelStatus()

    /// Tunnel state.
    var state: TunnelState = .disconnected

    var description: String {
        var s = "\(state), network "

        if packetTunnelStatus.isNetworkReachable {
            s += "reachable"
        } else {
            s += "unreachable"
        }

        if let connectingDate = packetTunnelStatus.connectingDate {
            s += ", started connecting at \(connectingDate.logFormatDate())"
        }

        return s
    }

    /// Updates the tunnel status from packet tunnel status, mapping relay to tunnel state.
    mutating func update(
        from packetTunnelStatus: PacketTunnelStatus,
        mappingRelayToState mapper: (PacketTunnelRelay?) -> TunnelState?
    ) {
        self.packetTunnelStatus = packetTunnelStatus

        if let newState = mapper(packetTunnelStatus.tunnelRelay) {
            state = newState
        }
    }

    /// Resets all fields to their defaults and assigns the next tunnel state.
    mutating func reset(to newState: TunnelState) {
        let currentRelay = packetTunnelStatus.tunnelRelay
        packetTunnelStatus = PacketTunnelStatus()
        packetTunnelStatus.tunnelRelay = currentRelay
        state = newState
    }
}

/// An enum that describes the tunnel state.
enum TunnelState: Equatable, CustomStringConvertible {
    /// Pending reconnect after disconnect.
    case pendingReconnect

    /// Connecting the tunnel.
    case connecting(_ relay: PacketTunnelRelay?)

    /// Connected the tunnel
    case connected(PacketTunnelRelay)

    /// Disconnecting the tunnel
    case disconnecting(ActionAfterDisconnect)

    /// Disconnected the tunnel
    case disconnected

    /// Reconnecting the tunnel. Normally this state appears in response to changing the
    /// relay constraints and asking the running tunnel to reload the configuration.
    case reconnecting(_ relay: PacketTunnelRelay)

    var description: String {
        switch self {
        case .pendingReconnect:
            return "pending reconnect after disconnect"
        case let .connecting(tunnelRelay):
            if let tunnelRelay = tunnelRelay {
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
        }
    }

    var isSecured: Bool {
        switch self {
        case .reconnecting, .connecting, .connected:
            return true
        case .pendingReconnect, .disconnecting, .disconnected:
            return false
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
