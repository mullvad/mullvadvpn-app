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
    func obfuscate(
        _ relays: REST.ServerRelaysResponse,
        using tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) -> RelayObfuscation {
        RelayObfuscation(
            allRelays: relays,
            obfuscatedRelays: filterQuicRelays(from: relays),
            port: .only(443),
            method: .quic
        )
    }

    private func filterQuicRelays(from relays: REST.ServerRelaysResponse) -> REST.ServerRelaysResponse {
        let filteredRelays = relays.wireguard.relays.filter { relay in
            let addressListIsEmpty = relay.features?.quic?.addrIn.isEmpty ?? true
            return !addressListIsEmpty
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
