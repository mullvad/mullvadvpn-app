// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadLogging
import MullvadSettings
import MullvadTypes
import Network

public protocol RelayPicking {
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
            portConstraint: derivePortConstraint(from: obfuscation, and: tunnelSettings),
            numberOfFailedAttempts: connectionAttemptCount,
            closeTo: location
        )

        // Resolve the addresses that the IP version preference allows
        let ipVersion = tunnelSettings.ipVersion
        let ipv4 =
            ipVersion.isIPv6 && !forceV4Address ? nil : resolveIpv4Endpoint(match: match, obfuscation: obfuscation)
        let ipv6 =
            ipVersion.isIPv4 || forceV4Address ? nil : resolveIpv6Endpoint(match: match, obfuscation: obfuscation)

        let socketAddress: SelectedSocketAddress
        switch (ipv4, ipv6) {
        case let (ipv4?, ipv6?): socketAddress = .both(ipv4, ipv6)
        case let (ipv4?, nil): socketAddress = .ipv4(ipv4)
        case let (nil, ipv6?): socketAddress = .ipv6(ipv6)
        case (nil, nil): throw NoRelaysSatisfyingConstraintsError(.noIPv6RelayFound)
        }

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

    private func resolveIpv4Endpoint(match: RelaySelectorMatch, obfuscation: RelayObfuscation?) -> IPv4Endpoint {
        let ipv4Address =
            if let obfuscation {
                applyObfuscatedIpAddresses(match: match, obfuscation: obfuscation)
            } else {
                match.endpoint.ipv4Relay.ip
            }
        return IPv4Endpoint(ip: ipv4Address, port: match.endpoint.ipv4Relay.port)
    }

    /// Returns `nil` if the relay cannot be reached over IPv6 with the obfuscation in use.
    private func resolveIpv6Endpoint(match: RelaySelectorMatch, obfuscation: RelayObfuscation?) -> IPv6Endpoint? {
        guard let ipv6Relay = match.endpoint.ipv6Relay else {
            return nil
        }

        let address: IPv6Address? =
            switch obfuscation?.method ?? .off {
            case .quic:
                // Regular IPv6 addresses do not work with QUIC.
                match.relay.features?.quic?.addrIn.compactMap({ IPv6Address($0) }).randomElement()
            case .shadowsocks:
                // Ports outside of the Shadowsocks port ranges only work with the extra addresses.
                if shadowsocksPortIsWithinRange(ipv6Relay.port, portRanges: relays.wireguard.shadowsocksPortRanges) {
                    shadowsocksExtraIpv6Address(in: match) ?? ipv6Relay.ip
                } else {
                    shadowsocksExtraIpv6Address(in: match)
                }
            case .off, .automatic, .on, .udpOverTcp, .lwo:
                ipv6Relay.ip
            }

        guard let address else {
            return nil
        }
        return IPv6Endpoint(ip: address, port: ipv6Relay.port)
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

    private func applyQuicIpAddress(in match: RelaySelectorMatch) -> IPv4Address {
        let defaultIpv4Address = match.endpoint.ipv4Relay.ip
        return match.relay.features?.quic?.addrIn.compactMap({ IPv4Address($0) }).randomElement() ?? defaultIpv4Address
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

    private func shadowsocksExtraIpv6Address(in match: RelaySelectorMatch) -> IPv6Address? {
        match.relay.shadowsocksExtraAddrIn?.compactMap({ IPv6Address($0) }).randomElement()
    }

    private func shadowsocksPortIsWithinRange(_ port: UInt16, portRanges: [[UInt16]]) -> Bool {
        let portRanges = RelaySelector.parseRawPortRanges(portRanges)
        return portRanges.contains(where: { $0.contains(port) })
    }

    private func derivePortConstraint(
        from obfuscation: RelayObfuscation?,
        and settings: LatestTunnelSettings
    ) -> RelayConstraint<UInt16> {
        obfuscation?.port ?? settings.relayConstraints.port
    }
}
