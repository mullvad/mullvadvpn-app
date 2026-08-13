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
import MullvadTypes

extension AsyncStream where Element == ObservedState {
    /// Consume the stream until `.disconnected` is observed or the stream ends.
    public func waitUntilDisconnected() async {
        for await newState in self {
            if case .disconnected = newState {
                return
            }
        }
    }
}

extension ObservedState {
    public var name: String {
        switch self {
        case .connected:
            "Connected"
        case .connecting:
            "Connecting"
        case .negotiatingEphemeralPeer:
            "Negotiating Post Quantum Secure Key"
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
        case let .connecting(connectionState),
            let .reconnecting(connectionState),
            let .connected(connectionState),
            let .negotiatingEphemeralPeer(connectionState, _),
            let .disconnecting(connectionState):
            connectionState
        default:
            nil
        }
    }

    /// Applies `body` to the associated connection state, preserving the current case.
    /// No-op in states that carry no connection state.
    public mutating func mutateConnectionState(_ body: (inout ObservedConnectionState) -> Void) {
        switch self {
        case var .connecting(connectionState):
            body(&connectionState)
            self = .connecting(connectionState)
        case var .reconnecting(connectionState):
            body(&connectionState)
            self = .reconnecting(connectionState)
        case var .connected(connectionState):
            body(&connectionState)
            self = .connected(connectionState)
        case .negotiatingEphemeralPeer(var connectionState, let privateKey):
            body(&connectionState)
            self = .negotiatingEphemeralPeer(connectionState, privateKey)
        case var .disconnecting(connectionState):
            body(&connectionState)
            self = .disconnecting(connectionState)
        case .initial, .disconnected, .error:
            break
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

extension ObservedConnectionState {
    mutating func incrementAttemptCount() {
        let (value, isOverflow) = connectionAttemptCount.addingReportingOverflow(1)
        connectionAttemptCount = isOverflow ? 0 : value
    }
}
