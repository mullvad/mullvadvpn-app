//
//  ObservedState+Testing.swift
//  PacketTunnelCoreTests
//
//  Created by Emīls on 2026-08-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

@testable import PacketTunnelCore

extension ObservedState {
    var isInitial: Bool {
        if case .initial = self { true } else { false }
    }

    var isConnecting: Bool {
        if case .connecting = self { true } else { false }
    }

    var isConnected: Bool {
        if case .connected = self { true } else { false }
    }

    var isReconnecting: Bool {
        if case .reconnecting = self { true } else { false }
    }

    var isDisconnected: Bool {
        if case .disconnected = self { true } else { false }
    }

    /// The reason the tunnel is blocked, or nil when it isn't.
    var blockedReason: BlockedStateReason? {
        blockedState?.reason
    }
}

/// Thread-safe sink for states observed from a `Task`, so a test can read them back afterwards.
public actor StateCollector {
    private var states: [ObservedState] = []

    func append(_ state: ObservedState) {
        states.append(state)
    }

    var collected: [ObservedState] {
        states
    }
}
