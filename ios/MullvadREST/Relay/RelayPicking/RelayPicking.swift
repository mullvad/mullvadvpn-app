//
//  RelaySelectorPicker.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2024-06-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import Network

protocol RelayPicking {
    var obfuscation: RelayObfuscation { get }
    var tunnelSettings: LatestTunnelSettings { get }
    var connectionAttemptCount: UInt { get }
    func pick() throws -> SelectedRelays
}

extension RelayPicking {
    func findBestMatch(
        from candidates: [RelayWithLocation<REST.ServerRelay>],
        closeTo location: Location? = nil,
        useObfuscatedPortIfAvailable: Bool
    ) throws -> SelectedRelay {
        var match = try RelaySelector.WireGuard.pickCandidate(
            from: candidates,
            wireguard: obfuscation.allRelays.wireguard,
            portConstraint: useObfuscatedPortIfAvailable
                ? obfuscation.port
                : tunnelSettings.relayConstraints.port,
            numberOfFailedAttempts: connectionAttemptCount,
            closeTo: location
        )

        if useObfuscatedPortIfAvailable && obfuscation.method == .shadowsocks {
            match = applyShadowsocksIpAddress(in: match)
        }

        return SelectedRelay(
            endpoint: match.endpoint,
            hostname: match.relay.hostname,
            location: match.location,
            features: match.relay.features
        )
    }

    private func applyShadowsocksIpAddress(in match: RelaySelectorMatch) -> RelaySelectorMatch {
        let port = match.endpoint.ipv4Relay.port
        let portRanges = RelaySelector.parseRawPortRanges(obfuscation.allRelays.wireguard.shadowsocksPortRanges)
        let portIsWithinRange = portRanges.contains(where: { $0.contains(port) })

        var endpoint = match.endpoint

        // If the currently selected obfuscation port is not within the allowed range (as specified
        // in the relay list), we should use one of the extra Shadowsocks IP addresses instead of
        // the default one.
        if !portIsWithinRange {
            var ipv4Address = match.endpoint.ipv4Relay.ip
            if let shadowsocksAddress = match.relay.shadowsocksExtraAddrIn?.randomElement() {
                ipv4Address = IPv4Address(shadowsocksAddress) ?? ipv4Address
            }

            endpoint = match.endpoint.override(ipv4Relay: IPv4Endpoint(
                ip: ipv4Address,
                port: port
            ))
        }

        return RelaySelectorMatch(endpoint: endpoint, relay: match.relay, location: match.location)
    }
}
