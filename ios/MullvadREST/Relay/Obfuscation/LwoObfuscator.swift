//
//  LwoObfuscator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-02-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

struct LwoObfuscator: RelayObfuscating {
    let relays: REST.ServerRelaysResponse
    let tunnelSettings: LatestTunnelSettings
    let connectionAttemptCount: UInt

    func obfuscate() throws -> RelayObfuscation {
        RelayObfuscation(
            allRelays: relays,
            obfuscatedRelays: filterLwoRelays(from: relays),
            port: try validateLwoPort(relays: relays, tunnelSettings: tunnelSettings),
            method: .lwo
        )
    }

    private func filterLwoRelays(from relays: REST.ServerRelaysResponse) -> REST.ServerRelaysResponse {
        return REST.ServerRelaysResponse(
            locations: relays.locations,
            wireguard: REST.ServerWireguardTunnels(
                ipv4Gateway: relays.wireguard.ipv4Gateway,
                ipv6Gateway: relays.wireguard.ipv6Gateway,
                portRanges: relays.wireguard.portRanges,
                relays: relays.wireguard.relays.filter { $0.supportsLwo },
                shadowsocksPortRanges: relays.wireguard.shadowsocksPortRanges
            ),
            bridge: relays.bridge
        )
    }

    private func validateLwoPort(
        relays: REST.ServerRelaysResponse,
        tunnelSettings: LatestTunnelSettings
    ) throws -> RelayConstraint<UInt16> {
        guard let customLwoPort = tunnelSettings.wireGuardObfuscation.lwoPort.portValue else {
            return .any
        }

        let portIsWithinValidWireGuardRanges = relays.wireguard.portRanges
            .contains { range in
                if let minPort = range.first, let maxPort = range.last {
                    return (minPort...maxPort).contains(customLwoPort)
                }
                return false
            }

        if !portIsWithinValidWireGuardRanges {
            throw NoRelaysSatisfyingConstraintsError(.invalidPort)
        }

        return .only(customLwoPort)
    }
}
