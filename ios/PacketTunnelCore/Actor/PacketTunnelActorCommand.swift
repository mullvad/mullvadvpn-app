// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
@preconcurrency import MullvadTypes
import Network

extension PacketTunnelActor {
    /// Describes events that the state machine handles. These can be user commands or non-user-initiated events
    enum Event: Sendable {
        /// Start tunnel.
        case start(StartOptions)

        /// Stop tunnel.
        case stop

        /// Reconnect tunnel.
        case reconnect(NextRelays, reason: ActorReconnectReason = .userInitiated)

        /// Enter blocked state.
        case error(BlockedStateReason)

        /// Notify that key rotation took place
        case notifyKeyRotated(Date?)

        /// Switch to using the recently pushed WG key.
        case switchKey

        /// Monitor events.
        case monitorEvent(_ event: TunnelMonitorEvent)

        /// Network reachability events.
        case networkReachability(Network.NWPath.Status)

        /// Update the device private key, as per post-quantum protocols
        case ephemeralPeerNegotiationStateChanged(EphemeralPeerNegotiationState, OneshotChannel)

        /// Notify that an ephemeral peer exchanging took place
        case notifyEphemeralPeerNegotiated

        /// Format command for log output.
        func logFormat() -> String {
            switch self {
            case .start:
                return "start"
            case .stop:
                return "stop"
            case let .reconnect(nextRelays, stopTunnelMonitor):
                switch nextRelays {
                case .current:
                    return "reconnect(current, \(stopTunnelMonitor))"
                case .random:
                    return "reconnect(random, \(stopTunnelMonitor))"
                case let .preSelected(selectedRelays):
                    return
                        "reconnect(\(selectedRelays.exit.hostname)\(selectedRelays.entry.flatMap { " via \($0.hostname)" } ?? ""), \(stopTunnelMonitor))"
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
            case .ephemeralPeerNegotiationStateChanged:
                return "ephemeralPeerNegotiationStateChanged"
            case .notifyEphemeralPeerNegotiated:
                return "notifyEphemeralPeerNegotiated"
            }
        }
    }
}
