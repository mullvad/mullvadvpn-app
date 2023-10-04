//
//  Command.swift
//  PacketTunnelCore
//
//  Created by pronebird on 27/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Describes action that actor can perform.
enum Command {
    /// Start tunnel.
    case start(StartOptions)

    /// Stop tunnel.
    case stop

    /// Reconnect tunnel.
    /// `stopTunnelMonitor = false` is only used when tunnel monitor is paused in response to connectivity loss and shouldn't be stopped explicitly,
    /// as this would reset its internal counters.
    case reconnect(NextRelay, stopTunnelMonitor: Bool = true)

    /// Enter blocked state.
    case error(BlockedStateReason)

    /// Notify that key rotation took place
    case notifyKeyRotated(Date?)

    /// Switch to using the recently pushed WG key.
    case switchKey

    /// Monitor events.
    case monitorEvent(_ event: TunnelMonitorEvent)

    /// Network reachability events.
    case networkReachability(NetworkPath)

    /// Format command for log output.
    func logFormat() -> String {
        switch self {
        case .start:
            return "start"
        case .stop:
            return "stop"
        case let .reconnect(nextRelay, stopTunnelMonitor):
            switch nextRelay {
            case .current:
                return "reconnect(current, \(stopTunnelMonitor))"
            case .random:
                return "reconnect(random, \(stopTunnelMonitor))"
            case let .preSelected(selectedRelay):
                return "reconnect(\(selectedRelay.hostname), \(stopTunnelMonitor))"
            }
        case let .error(reason):
            return "error(\(reason))"
        case .notifyKeyRotated:
            return "notifyKeyRotated"
        case let .monitorEvent(event):
            switch event {
            case .connectionEstablished:
                return "monitorEvent(connectionEstablished)"
            case .connectionLost:
                return "monitorEvent(connectionLost)"
            }
        case .networkReachability:
            return "networkReachability"
        case .switchKey:
            return "switchKey"
        }
    }
}
