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
    var relays: REST.ServerRelaysResponse { get }
    var tunnelSettings: LatestTunnelSettings { get }
    var connectionAttemptCount: UInt { get }
    func pick() throws -> SelectedRelays
}

extension RelayPicking {
    func findBestMatch(
        from candidates: [RelayWithLocation<REST.ServerRelay>],
        closeTo location: Location? = nil,
        obfuscation: RelayObfuscation?,
        forceV4Address: Bool = false,
    ) throws -> SelectedRelay {
        let match = try RelaySelector.WireGuard.pickCandidate(
            from: candidates,
            wireguard: relays.wireguard,
            portConstraint: obfuscation?.port ?? tunnelSettings.relayConstraints.port,
            numberOfFailedAttempts: connectionAttemptCount,
            ipVersion: tunnelSettings.ipVersion,
            closeTo: location
        )

        // Resolve the socket address based on IP version preference
        let socketAddress = try resolveSocketAddress(
            match: match,
            obfuscation: obfuscation,
            forceV4: forceV4Address,
        )

        // Convert WireGuardObfuscationState to ObfuscationMethod
        let obfuscationMethod = try resolveObfuscationMethod(
            features: match.relay.features,
            obfuscation: obfuscation
        )

        let selectedEndpoint = SelectedEndpoint(
            socketAddress: socketAddress,
            ipv4Gateway: match.endpoint.ipv4Gateway,
            ipv6Gateway: match.endpoint.ipv6Gateway,
            publicKey: match.endpoint.publicKey,
            obfuscation: obfuscationMethod,
        )

        return SelectedRelay(
            endpoint: selectedEndpoint,
            hostname: match.relay.hostname,
            location: match.location,
            isIPOverridden: match.relay.isIPOverridden ?? false,
            features: match.relay.features
        )
    }

    /// Resolves a single socket address based on IP version preference and obfuscation settings.
    /// Throws an error if IPv6 is required but no IPv6 endpoint is available.
    private func resolveSocketAddress(
        match: RelaySelectorMatch,
        obfuscation: RelayObfuscation?,
        forceV4: Bool,
    ) throws -> AnyIPEndpoint {
        // Try IPv6 first if preferred and available
        if tunnelSettings.ipVersion.isIPv6, !forceV4 {
            guard let ipv6Relay = match.endpoint.ipv6Relay else {
                throw NoRelaysSatisfyingConstraintsError(.noIPv6RelayFound)
            }

            let ipv6Address: IPv6Address
            if let obfuscation {
                guard let obfuscatedIpv6 = applyObfuscatedIpV6Addresses(match: match, obfuscation: obfuscation) else {
                    throw NoRelaysSatisfyingConstraintsError(.noIPv6RelayFound)
                }
                ipv6Address = obfuscatedIpv6
            } else {
                ipv6Address = ipv6Relay.ip
            }
            return .ipv6(IPv6Endpoint(ip: ipv6Address, port: ipv6Relay.port))
        }

        // Fall back to IPv4
        let ipv4Address =
            if let obfuscation {
                applyObfuscatedIpAddresses(match: match, obfuscation: obfuscation)
            } else {
                match.endpoint.ipv4Relay.ip
            }
        return .ipv4(IPv4Endpoint(ip: ipv4Address, port: match.endpoint.ipv4Relay.port))
    }

    /// Converts WireGuardObfuscationState to ObfuscationMethod.
    private func resolveObfuscationMethod(
        features: REST.ServerRelay.Features?,
        obfuscation: RelayObfuscation?
    ) throws -> ObfuscationMethod {
        return switch obfuscation?.method ?? .off {
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
                logger.error(
                    "Relay should support QUIC, but config cannot be read from relay features. This is probably a bug."
                )
                throw NoRelaysSatisfyingConstraintsError(.relayConstraintNotMatching)
            }
        case .lwo:
            .lwo
        }
    }

    private func applyObfuscatedIpAddresses(match: RelaySelectorMatch, obfuscation: RelayObfuscation) -> IPv4Address {
        switch obfuscation.method {
        case .shadowsocks:
            applyShadowsocksIpAddress(in: match, portRanges: relays.wireguard.shadowsocksPortRanges)
        case .quic:
            applyQuicIpAddress(in: match)
        case .off, .automatic, .on, .udpOverTcp, .lwo:
            match.endpoint.ipv4Relay.ip
        }
    }

    private func applyObfuscatedIpV6Addresses(
        match: RelaySelectorMatch,
        obfuscation: RelayObfuscation
    ) -> IPv6Address? {
        switch obfuscation.method {
        case .shadowsocks:
            applyShadowsocksIpv6Address(in: match)
        case .quic:
            applyQuicIpv6Address(in: match)
        case .off, .automatic, .on, .udpOverTcp, .lwo:
            match.endpoint.ipv6Relay?.ip
        }
    }

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

    private func applyShadowsocksIpAddress(in match: RelaySelectorMatch, portRanges: [[UInt16]]) -> IPv4Address {
        let defaultIpv4Address = match.endpoint.ipv4Relay.ip
        let extraAddresses = match.relay.shadowsocksExtraAddrIn?.compactMap({ IPv4Address($0) }) ?? []

        // If the currently selected obfuscation port is not within the allowed range (as specified
        // in the relay list), we should use only one of the extra Shadowsocks IPs instead of the
        // default one.
        return
            if !shadowsocksPortIsWithinRange(
                match.endpoint.ipv4Relay.port,
                portRanges: portRanges
            )
        {
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

        guard match.endpoint.ipv6Relay?.port != nil else {
            return extraAddresses.randomElement()
        }
        if !extraAddresses.isEmpty {
            return extraAddresses.randomElement()!
        }
        return defaultIpv6Address
    }

    private func shadowsocksPortIsWithinRange(_ port: UInt16, portRanges: [[UInt16]]) -> Bool {
        let portRanges = RelaySelector.parseRawPortRanges(portRanges)
        return portRanges.contains(where: { $0.contains(port) })
    }
}
