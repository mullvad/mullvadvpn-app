//
//  ObfuscationMethodSelector.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2024-11-01.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

public protocol ObfuscationNotSupportedBypassing {
    func bypassUnsupportedObfuscation(_: WireGuardObfuscationState) -> WireGuardObfuscationState
}

public struct ObfuscationMethodSelector {
    /// This retry logic used is explained at the following link:
    /// https://github.com/mullvad/mullvadvpn-app/blob/main/docs/relay-selector.md#default-constraints-for-tunnel-endpoints
    ///
    /// - Note: This method should never return `.automatic`.
    public static func obfuscationMethodBy(
        connectionAttemptCount: UInt,
        tunnelSettings: LatestTunnelSettings,
        obfuscationBias: any ObfuscationNotSupportedBypassing,
    ) -> WireGuardObfuscationState {
        let selectedObfuscation: WireGuardObfuscationState =
            if tunnelSettings.wireGuardObfuscation.state == .automatic {
                if connectionAttemptCount.isOrdered(nth: 2, forEverySetOf: 4) {
                    .shadowsocks
                } else if connectionAttemptCount.isOrdered(nth: 3, forEverySetOf: 4) {
                    .quic
                } else if connectionAttemptCount.isOrdered(nth: 4, forEverySetOf: 4) {
                    .udpOverTcp
                } else {
                    .off
                }
            } else {
                tunnelSettings.wireGuardObfuscation.state
            }
        return obfuscationBias.bypassUnsupportedObfuscation(selectedObfuscation)
    }
}

public struct UnsupportedObfuscationBypass: ObfuscationNotSupportedBypassing {
    let relayConstraint: RelayConstraint<UserSelectedRelays>
    let relays: REST.ServerRelaysResponse
    let filterConstraint: RelayConstraint<RelayFilter>
    let daitaEnabled: Bool

    public init(
        relayConstraint: RelayConstraint<UserSelectedRelays>,
        relays: REST.ServerRelaysResponse,
        filterConstraint: RelayConstraint<RelayFilter>,
        daitaEnabled: Bool
    ) {
        self.relayConstraint = relayConstraint
        self.relays = relays
        self.filterConstraint = filterConstraint
        self.daitaEnabled = daitaEnabled
    }

    public func bypassUnsupportedObfuscation(_ obfuscation: WireGuardObfuscationState) -> WireGuardObfuscationState {
        do {
            let candidates = try RelaySelector.WireGuard.findCandidates(
                by: relayConstraint,
                in: relays,
                filterConstraint: filterConstraint,
                daitaEnabled: daitaEnabled
            )
            return candidates.isEmpty ? .shadowsocks : obfuscation
        } catch {
            return .shadowsocks
        }
    }
}

public struct UnbiasedObfuscationBypass: ObfuscationNotSupportedBypassing {
    public init() {}
    public func bypassUnsupportedObfuscation(_ obfuscation: WireGuardObfuscationState) -> WireGuardObfuscationState {
        obfuscation
    }
}
