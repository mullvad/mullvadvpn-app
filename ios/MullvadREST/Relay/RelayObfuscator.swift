//
//  ObfuscatorPortSelector.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-01.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

public struct RelayObfuscation {
    let allRelays: REST.ServerRelaysResponse
    let obfuscatedRelays: REST.ServerRelaysResponse
    let port: RelayConstraint<UInt16>
    var method: WireGuardObfuscationState
}

struct RelayObfuscator {
    let relays: REST.ServerRelaysResponse

    func obfuscate(
        tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) throws -> RelayObfuscation {
        var obfuscatedRelays = relays
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
            obfuscatedRelays = obfuscateShadowsocksRelays(tunnelSettings: tunnelSettings)
            port = obfuscateShadowsocksPort(
                tunnelSettings: tunnelSettings,
                shadowsocksPortRanges: relays.wireguard.shadowsocksPortRanges
            )
        case .quic:
            obfuscatedRelays = obfuscateQUICRelays(tunnelSettings: tunnelSettings)
            port = .only(443)
        default:
            break
        }

        return RelayObfuscation(
            allRelays: relays,
            obfuscatedRelays: obfuscatedRelays,
            port: port,
            method: obfuscationMethod
        )
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
        for port: WireGuardObfuscationShadowsocksPort
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

    private func obfuscateQUICRelays(tunnelSettings: LatestTunnelSettings) -> REST.ServerRelaysResponse {
        let relays = relays
        let wireGuardObfuscation = tunnelSettings.wireGuardObfuscation

        return wireGuardObfuscation.state == .quic
            ? filterQUICRelays(from: relays)
            : relays
    }

    private func filterQUICRelays(from relays: REST.ServerRelaysResponse) -> REST.ServerRelaysResponse {
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

    private func obfuscateUdpOverTcpPort(
        tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) -> RelayConstraint<UInt16> {
        switch tunnelSettings.wireGuardObfuscation.udpOverTcpPort {
        case .automatic:
            return [.only(80), .only(5001)].randomElement()!
        case .port5001:
            return .only(5001)
        case .port80:
            return .only(80)
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
