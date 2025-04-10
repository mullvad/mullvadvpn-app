//
//  ObfuscationMethodSelector.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2024-11-01.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

public struct ObfuscationMethodSelector {
    /// This retry logic used is explained at the following link:
    /// https://github.com/mullvad/mullvadvpn-app/blob/main/docs/relay-selector.md#default-constraints-for-tunnel-endpoints
    public static func obfuscationMethodBy(
        connectionAttemptCount: UInt,
        tunnelSettings: LatestTunnelSettings
    ) -> WireGuardObfuscationState {
        // TODO: Revisit this when QUIC obfuscation is added
        if tunnelSettings.wireGuardObfuscation.state == .automatic {
            if connectionAttemptCount.isOrdered(nth: 3, forEverySetOf: 4) {
                .shadowsocks
            } else if connectionAttemptCount.isOrdered(nth: 4, forEverySetOf: 4) {
                .udpOverTcp
            } else {
                .off
            }
        } else {
            tunnelSettings.wireGuardObfuscation.state
        }
    }
}
