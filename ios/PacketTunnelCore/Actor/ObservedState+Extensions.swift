//
//  ObservedState+Extensions.swift
//  PacketTunnelCore
//
//  Created by pronebird on 16/10/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
