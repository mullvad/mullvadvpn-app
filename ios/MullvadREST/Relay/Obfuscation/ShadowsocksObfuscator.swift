//
//  ShadowsocksObfuscator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-09-04.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

struct ShadowsocksObfuscator: RelayObfuscating {
    let relays: REST.ServerRelaysResponse
    let tunnelSettings: LatestTunnelSettings
    let connectionAttemptCount: UInt

    func obfuscate() -> RelayObfuscation {
        RelayObfuscation(
            allRelays: relays,
            obfuscatedRelays: filterShadowsocksRelays(
                from: relays,
                for: tunnelSettings.wireGuardObfuscation.shadowsocksPort
            ),
            port: obfuscateShadowsocksPort(
                tunnelSettings: tunnelSettings,
                shadowsocksPortRanges: relays.wireguard.shadowsocksPortRanges
            ),
            method: .shadowsocks
        )
    }

    private func filterShadowsocksRelays(
        from relays: REST.ServerRelaysResponse,
        for port: WireGuardObfuscationShadowsocksPort
    ) -> REST.ServerRelaysResponse {
        let portRanges = RelaySelector.parseRawPortRanges(relays.wireguard.shadowsocksPortRanges)

        // If the selected port is within the shadowsocks port ranges we can select from all relays.
        // Standard ports can use regular ipv6AddrIn for IPv6.
        guard
            case let .custom(port) = port,
            !portRanges.contains(where: { $0.contains(port) })
        else {
            return relays
        }

        // Custom port outside standard ranges - require shadowsocksExtraAddrIn.
        // If IPv6 is enabled, also require IPv6 addresses in shadowsocksExtraAddrIn.
        let filteredRelays: [REST.ServerRelay]
        if tunnelSettings.ipVersion.isIPv6 {
            filteredRelays = relays.wireguard.relays.filter { $0.hasShadowsocksIpv6 }
        } else {
            filteredRelays = relays.wireguard.relays.filter { $0.shadowsocksExtraAddrIn != nil }
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

    private func obfuscateShadowsocksPort(
        tunnelSettings: LatestTunnelSettings,
        shadowsocksPortRanges: [[UInt16]]
    ) -> RelayConstraint<UInt16> {
        let wireGuardObfuscation = tunnelSettings.wireGuardObfuscation

        let shadowsockPort: () -> UInt16? = {
            switch wireGuardObfuscation.shadowsocksPort {
            case let .custom(port):
                port
            default:
                RelaySelector.pickRandomPort(rawPortRanges: shadowsocksPortRanges)
            }
        }

        guard let port = shadowsockPort() else {
            return tunnelSettings.relayConstraints.port
        }

        return .only(port)
    }
}
