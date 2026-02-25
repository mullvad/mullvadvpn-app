//
//  RelaySelectorPicker.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2024-06-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadSettings
import MullvadTypes
import Network

protocol RelayPicking {
    var logger: Logger { get }
    var obfuscation: RelayObfuscation { get }
    var tunnelSettings: LatestTunnelSettings { get }
    var connectionAttemptCount: UInt { get }
    func pick() throws -> SelectedRelays
}

extension RelayPicking {
    func findBestMatch(
        from candidates: [RelayWithLocation<REST.ServerRelay>],
        closeTo location: Location? = nil,
        applyObfuscatedIps: Bool,
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

        // Resolve the socket address based on IP version preference
        let socketAddress = resolveSocketAddress(
            match: match,
            applyObfuscatedIps: applyObfuscatedIps,
        )

        // Convert WireGuardObfuscationState to ObfuscationMethod
        let obfuscationMethod = try resolveObfuscationMethod(features: match.relay.features)

        let selectedEndpoint = SelectedEndpoint(
            socketAddress: socketAddress,
            ipv4Gateway: match.endpoint.ipv4Gateway,
            ipv6Gateway: match.endpoint.ipv6Gateway,
            publicKey: match.endpoint.publicKey,
            obfuscation: obfuscationMethod
        )

        return SelectedRelay(
            endpoint: selectedEndpoint,
            hostname: match.relay.hostname,
            location: match.location,
            features: match.relay.features
        )
    }

    /// Resolves a single socket address based on IP version preference and obfuscation settings.
    private func resolveSocketAddress(
        match: RelaySelectorMatch,
        applyObfuscatedIps: Bool,
    ) -> AnyIPEndpoint {
        let ipv4Address =
            if applyObfuscatedIps {
                applyObfuscatedIpAddresses(match: match)
            } else {
                match.endpoint.ipv4Relay.ip
            }
        return .ipv4(IPv4Endpoint(ip: ipv4Address, port: match.endpoint.ipv4Relay.port))
    }

    /// Converts WireGuardObfuscationState to ObfuscationMethod.
    private func resolveObfuscationMethod(features: REST.ServerRelay.Features?) throws -> ObfuscationMethod {
        switch obfuscation.method {
        case .off, .automatic:
            .off
        case .on:
            // `.on` is a legacy state that shouldn't occur in practice
            .off
        case .udpOverTcp:
            .udpOverTcp
        case .shadowsocks:
            .shadowsocks
        case .quic:
            if let quicFeatures = features?.quic {
                .quic(hostname: quicFeatures.domain, token: quicFeatures.token)
            } else {
                logger.error("Invalid QUIC config")
                throw NoRelaysSatisfyingConstraintsError(.relayConstraintNotMatching)
            }
        case .lwo:
            .lwo
        }
    }

    private func applyObfuscatedIpAddresses(match: RelaySelectorMatch) -> IPv4Address {
        switch obfuscation.method {
        case .shadowsocks:
            applyShadowsocksIpAddress(in: match)
        case .quic:
            applyQuicIpAddress(in: match)
        case .off, .automatic, .on, .udpOverTcp, .lwo:
            match.endpoint.ipv4Relay.ip
        }
    }

    private func applyQuicIpAddress(in match: RelaySelectorMatch) -> IPv4Address {
        let defaultIpv4Address = match.endpoint.ipv4Relay.ip
        return match.relay.features?.quic?.addrIn.compactMap({ IPv4Address($0) }).randomElement() ?? defaultIpv4Address
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

    private func shadowsocksPortIsWithinRange(_ port: UInt16) -> Bool {
        let portRanges = RelaySelector.parseRawPortRanges(obfuscation.allRelays.wireguard.shadowsocksPortRanges)
        return portRanges.contains(where: { $0.contains(port) })
    }
}
