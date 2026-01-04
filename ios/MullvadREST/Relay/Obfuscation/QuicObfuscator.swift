//
//  QuicObfuscator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-09-04.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

struct QuicObfuscator: RelayObfuscating {
    let relays: REST.ServerRelaysResponse
    let tunnelSettings: LatestTunnelSettings
    let connectionAttemptCount: UInt

    func obfuscate() -> RelayObfuscation {
        RelayObfuscation(
            allRelays: relays,
            obfuscatedRelays: filterQuicRelays(from: relays),
            port: .only(443),
            method: .quic
        )
    }

    private func filterQuicRelays(from relays: REST.ServerRelaysResponse) -> REST.ServerRelaysResponse {
        var filteredRelays = relays.wireguard.relays.filter { $0.supportsQuic }

        // If IPv6 is required, filter to relays with QUIC IPv6 addresses
        // Regular entry IPv6 addresses don't work with QUIC
        if tunnelSettings.ipVersion.isIPv6 {
            filteredRelays = filteredRelays.filter { $0.hasQuicIpv6 }
        }

        return REST.ServerRelaysResponse(
            locations: relays.locations,
            wireguard: REST.ServerWireguardTunnels(
                ipv4Gateway: relays.wireguard.ipv4Gateway,
                ipv6Gateway: relays.wireguard.ipv6Gateway,
                portRanges: relays.wireguard.portRanges,
                relays: filteredRelays,
                shadowsocksPortRanges: relays.wireguard.shadowsocksPortRanges
            ),
            bridge: relays.bridge
        )
    }
}
