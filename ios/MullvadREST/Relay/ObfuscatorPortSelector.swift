//
//  ObfuscatorPortSelector.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-01.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

struct ObfuscatorPortSelectorResult {
    let relays: REST.ServerRelaysResponse
    let port: RelayConstraint<UInt16>
}

struct ObfuscatorPortSelector {
    let relays: REST.ServerRelaysResponse

    func obfuscate(
        tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) throws -> ObfuscatorPortSelectorResult {
        var relays = relays
        var port = tunnelSettings.relayConstraints.port
        let obfuscationMethod = ObfuscationMethodSelector.obfuscationMethodBy(
            connectionAttemptCount: connectionAttemptCount,
            tunnelSettings: tunnelSettings
        )

        switch obfuscationMethod {
        case .udpOverTcp:
            port = obfuscateUdpOverTcpPort(
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            )
        case .shadowsocks:
            relays = obfuscateShadowsocksRelays(tunnelSettings: tunnelSettings)
            port = obfuscateShadowsocksPort(
                tunnelSettings: tunnelSettings,
                shadowsocksPortRanges: relays.wireguard.shadowsocksPortRanges
            )
        default:
            break
        }

        return ObfuscatorPortSelectorResult(relays: relays, port: port)
    }

    private func obfuscateShadowsocksRelays(tunnelSettings: LatestTunnelSettings) -> REST.ServerRelaysResponse {
        let relays = relays
        let wireGuardObfuscation = tunnelSettings.wireGuardObfuscation

        return wireGuardObfuscation.state == .shadowsocks
            ? filterShadowsocksRelays(from: relays, for: wireGuardObfuscation.shadowsocksPort)
            : relays
    }

    private func filterShadowsocksRelays(
        from relays: REST.ServerRelaysResponse,
        for port: WireGuardObfuscationShadowsockPort
    ) -> REST.ServerRelaysResponse {
        let portRanges = RelaySelector.parseRawPortRanges(relays.wireguard.shadowsocksPortRanges)

        // If the selected port is within the shadowsocks port ranges we can select from all relays.
        guard
            case let .custom(port) = port,
            !portRanges.contains(where: { $0.contains(port) })
        else {
            return relays
        }

        let filteredRelays = relays.wireguard.relays.filter { relay in
            relay.shadowsocksExtraAddrIn != nil
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

    private func obfuscateUdpOverTcpPort(
        tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) -> RelayConstraint<UInt16> {
        switch tunnelSettings.wireGuardObfuscation.udpOverTcpPort {
        case .automatic:
            (connectionAttemptCount % 2 == 0) ? .only(80) : .only(5001)
        case .port5001:
            .only(5001)
        case .port80:
            .only(80)
        }
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

        guard
            wireGuardObfuscation.state == .shadowsocks,
            let port = shadowsockPort()
        else {
            return tunnelSettings.relayConstraints.port
        }

        return .only(port)
    }
}
