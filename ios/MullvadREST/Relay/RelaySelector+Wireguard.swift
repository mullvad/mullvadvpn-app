//
//  RelaySelector+Wireguard.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-05-17.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import CoreLocation
import MullvadSettings
import MullvadTypes

extension RelaySelector {
    public enum WireGuard {
        /// Filters relay list using given constraints.
        public static func findCandidates(
            by relayConstraint: RelayConstraint<UserSelectedRelays>,
            in relays: REST.ServerRelaysResponse,
            filterConstraint: RelayConstraint<RelayFilter>,
            daitaEnabled: Bool
        ) throws -> [RelayWithLocation<REST.ServerRelay>] {
            let mappedRelays = RelayWithLocation.locateRelays(
                relays: relays.wireguard.relays,
                locations: relays.locations
            )

            return try applyConstraints(
                relayConstraint,
                filterConstraint: filterConstraint,
                daitaEnabled: daitaEnabled,
                relays: mappedRelays
            )
        }

        /// Picks a random relay from a list.
        public static func pickCandidate(
            from relayWithLocations: [RelayWithLocation<REST.ServerRelay>],
            wireguard: REST.ServerWireguardTunnels,
            portConstraint: RelayConstraint<UInt16>,
            numberOfFailedAttempts: UInt,
            closeTo referenceLocation: Location? = nil
        ) throws -> RelaySelectorMatch {
            let port = try evaluatePort(
                portConstraint: portConstraint,
                rawPortRanges: wireguard.portRanges,
                numberOfFailedAttempts: numberOfFailedAttempts
            )

            var relayWithLocation: RelayWithLocation<REST.ServerRelay>?
            if let referenceLocation {
                let relay =
                    closestRelay(
                        to: CLLocationCoordinate2D(
                            latitude: referenceLocation.latitude, longitude: referenceLocation.longitude),
                        using: relayWithLocations
                    ) as? REST.ServerRelay
                relayWithLocation = relayWithLocations.first(where: { $0.relay == relay })
            }

            guard
                let relayWithLocation = relayWithLocation ?? pickRandomRelayByWeight(relays: relayWithLocations)
            else {
                throw NoRelaysSatisfyingConstraintsError(.relayConstraintNotMatching)
            }

            return createMatch(for: relayWithLocation, port: port, wireguard: wireguard)
        }
    }

    private static func evaluatePort(
        portConstraint: RelayConstraint<UInt16>,
        rawPortRanges: [[UInt16]],
        numberOfFailedAttempts: UInt
    ) throws -> UInt16 {
        let port = applyPortConstraint(
            portConstraint,
            rawPortRanges: rawPortRanges,
            numberOfFailedAttempts: numberOfFailedAttempts
        )

        guard let port else {
            throw NoRelaysSatisfyingConstraintsError(.invalidPort)
        }

        return port
    }

    private static func createMatch(
        for relayWithLocation: RelayWithLocation<REST.ServerRelay>,
        port: UInt16,
        wireguard: REST.ServerWireguardTunnels
    ) -> RelaySelectorMatch {
        let endpoint = MullvadEndpoint(
            ipv4Relay: IPv4Endpoint(
                ip: relayWithLocation.relay.ipv4AddrIn,
                port: port
            ),
            ipv6Relay: nil,
            ipv4Gateway: wireguard.ipv4Gateway,
            ipv6Gateway: wireguard.ipv6Gateway,
            publicKey: relayWithLocation.relay.publicKey
        )

        return RelaySelectorMatch(
            endpoint: endpoint,
            relay: relayWithLocation.relay,
            location: relayWithLocation.serverLocation
        )
    }
}
