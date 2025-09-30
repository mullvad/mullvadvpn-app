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
        applyObfuscatedIps: Bool
    ) throws -> SelectedRelay {
        let match = try RelaySelector.WireGuard.pickCandidate(
            from: candidates,
            wireguard: obfuscation.allRelays.wireguard,
            portConstraint: applyObfuscatedIps
                ? obfuscation.port
                : tunnelSettings.relayConstraints.port,
            numberOfFailedAttempts: connectionAttemptCount,
            closeTo: location
        )

        var ipv4Address = match.endpoint.ipv4Relay.ip
        if applyObfuscatedIps {
            switch obfuscation.method {
            // If the currently selected obfuscation port is not within the allowed range (as specified
            // in the relay list), we should use one of the extra Shadowsocks IP addresses instead of
            // the default one.
            case .shadowsocks where !shadowsocksPortIsWithinRange(match.endpoint.ipv4Relay.port):
                ipv4Address = applyShadowsocksIpAddress(in: match)
            case .quic:
                ipv4Address = applyQuicIpAddress(in: match)
            case .automatic, .off, .on, .udpOverTcp, .shadowsocks:
                break
            }
        }

        return SelectedRelay(
            endpoint: match.endpoint.override(ipv4Relay: IPv4Endpoint(
                ip: ipv4Address,
                port: match.endpoint.ipv4Relay.port
            )),
            hostname: match.relay.hostname,
            location: match.location,
            features: match.relay.features
        )
    }

    private func shadowsocksPortIsWithinRange(_ port: UInt16) -> Bool {
        let portRanges = RelaySelector.parseRawPortRanges(obfuscation.allRelays.wireguard.shadowsocksPortRanges)
        return portRanges.contains(where: { $0.contains(port) })
    }

    private func applyShadowsocksIpAddress(in match: RelaySelectorMatch) -> IPv4Address {
        let ipv4Address = match.endpoint.ipv4Relay.ip

        return if let shadowsocksAddress = match.relay.shadowsocksExtraAddrIn?.randomElement() {
            IPv4Address(shadowsocksAddress) ?? ipv4Address
        } else {
            ipv4Address
        }
    }

    private func applyQuicIpAddress(in match: RelaySelectorMatch) -> IPv4Address {
        let ipv4Address = match.endpoint.ipv4Relay.ip

        return if let quicAddress = match.relay.features?.quic?.addrIn.randomElement() {
            IPv4Address(quicAddress) ?? ipv4Address
        } else {
            ipv4Address
        }
    }
}
