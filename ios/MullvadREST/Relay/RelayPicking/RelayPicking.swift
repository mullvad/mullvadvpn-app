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
            ipVersion: tunnelSettings.ipVersion,
            closeTo: location
        )


        let v6Endpoint: IPv6Endpoint? = if let v6Relay = match.endpoint.ipv6Relay { IPv6Endpoint(ip: applyObfuscatedIps ? applyObfuscatedIpV6Addresses(match: match) ?? v6Relay.ip : v6Relay.ip, port: v6Relay.port) } else {
            nil
        };
        return SelectedRelay(
            endpoint: match.endpoint.override(
                ipv4Relay: IPv4Endpoint(
                    ip: applyObfuscatedIps
                        ? applyObfuscatedIpAddresses(match: match)
                        : match.endpoint.ipv4Relay.ip,
                    port: match.endpoint.ipv4Relay.port
                ),
                ipv6Relay: v6Endpoint,

            ),
            hostname: match.relay.hostname,
            location: match.location,
            features: match.relay.features
        )
    }

    private func applyObfuscatedIpAddresses(match: RelaySelectorMatch) -> IPv4Address {
        switch obfuscation.method {
        case .shadowsocks:
            applyShadowsocksIpAddress(in: match)
        case .quic:
            applyQuicIpAddress(in: match)
        case .off, .automatic, .on, .udpOverTcp:
            match.endpoint.ipv4Relay.ip
        }
    }

    private func applyObfuscatedIpV6Addresses(match: RelaySelectorMatch) -> IPv6Address? {

        switch obfuscation.method {
        case .shadowsocks:
            applyShadowsocksIpv6Address(in: match)
        case .quic:
            applyQuicIpv6Address(in: match)
        case .off, .automatic, .on, .udpOverTcp:
            match.endpoint.ipv6Relay?.ip
        }
    }

//    private func pickIpForEndpoint(endoint: MullvadEndpoint) -> AnyIPAddress {
//         tunnelSettings.ipVersion.isIPv6 {
//
//        }
//    }

    private func applyQuicIpAddress(in match: RelaySelectorMatch) -> IPv4Address {
        let defaultIpv4Address = match.endpoint.ipv4Relay.ip
        return match.relay.features?.quic?.addrIn.compactMap({ IPv4Address($0) }).randomElement() ?? defaultIpv4Address
    }

    private func applyQuicIpv6Address(in match: RelaySelectorMatch) -> IPv6Address? {
        let defaultIpv6Address = match.endpoint.ipv6Relay?.ip
        return match.relay.features?.quic?.addrIn
            .compactMap({ IPv6Address($0) })
            .randomElement() ?? defaultIpv6Address
    }

    private func applyShadowsocksIpAddress(in match: RelaySelectorMatch) -> IPv4Address {
        let defaultIpv4Address = match.endpoint.ipv4Relay.ip
        let extraAddresses = match.relay.shadowsocksExtraAddrIn?.compactMap({ IPv4Address($0) }) ?? []

        // If the currently selected obfuscation port is not within the allowed range (as specified
        // in the relay list), we should use only one of the extra Shadowsocks IPs instead of the
        // default one.
        return if !shadowsocksPortIsWithinRange(match.endpoint.ipv4Relay.port) {
            extraAddresses.randomElement() ?? defaultIpv4Address
        } else {
            // If the currently selected obfuscation port is within the allowed range we select from
            // a mix of extra addresses and the default IP.
            // Safe to unwrap since array is never empty.
            (extraAddresses + [defaultIpv4Address]).randomElement().unsafelyUnwrapped
        }
    }

    private func applyShadowsocksIpv6Address(in match: RelaySelectorMatch) -> IPv6Address? {
        let defaultIpv6Address = match.endpoint.ipv6Relay?.ip
        let extraAddresses = match.relay.shadowsocksExtraAddrIn?.compactMap({ IPv6Address($0) }) ?? []

        guard let port = match.endpoint.ipv6Relay?.port else {
            return nil
        }
        return if !shadowsocksPortIsWithinRange(port) {
            extraAddresses.randomElement() ?? defaultIpv6Address
        } else {
            extraAddresses.randomElement()
        }
    }


    private func shadowsocksPortIsWithinRange(_ port: UInt16) -> Bool {
        let portRanges = RelaySelector.parseRawPortRanges(obfuscation.allRelays.wireguard.shadowsocksPortRanges)
        return portRanges.contains(where: { $0.contains(port) })
    }
}
