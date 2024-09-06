//
//  RelaySelector+Wireguard.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-05-17.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

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
            let mappedRelays = mapRelays(relays: relays.wireguard.relays, locations: relays.locations)

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
            relays: REST.ServerRelaysResponse,
            portConstraint: RelayConstraint<UInt16>,
            numberOfFailedAttempts: UInt,
            closeTo referenceLocation: Location? = nil
        ) throws -> RelaySelectorMatch {
            let port = try evaluatePort(
                relays: relays,
                portConstraint: portConstraint,
                numberOfFailedAttempts: numberOfFailedAttempts
            )

            var relayWithLocation: RelayWithLocation<REST.ServerRelay>?
            if let referenceLocation {
                let relay = closestRelay(to: referenceLocation, using: relayWithLocations)
                relayWithLocation = relayWithLocations.first(where: { $0.relay == relay })
            }

            guard
                let relayWithLocation = relayWithLocation ?? pickRandomRelayByWeight(relays: relayWithLocations)
            else {
                throw NoRelaysSatisfyingConstraintsError(.relayConstraintNotMatching)
            }

            return createMatch(for: relayWithLocation, port: port, relays: relays)
        }

        public static func closestRelay(
            to location: Location,
            using relayWithLocations: [RelayWithLocation<REST.ServerRelay>]
        ) -> REST.ServerRelay? {
            let relaysWithDistance = relayWithLocations.map {
                RelayWithDistance(
                    relay: $0.relay,
                    distance: Haversine.distance(
                        location.latitude,
                        location.longitude,
                        $0.serverLocation.latitude,
                        $0.serverLocation.longitude
                    )
                )
            }.sorted {
                $0.distance < $1.distance
            }.prefix(5)

            let relaysGroupedByDistance = Dictionary(grouping: relaysWithDistance, by: { $0.distance })
            guard let closetsRelayGroup = relaysGroupedByDistance.min(by: { $0.key < $1.key })?.value else {
                return nil
            }

            var greatestDistance = 0.0
            closetsRelayGroup.forEach {
                if $0.distance > greatestDistance {
                    greatestDistance = $0.distance
                }
            }

            let closestRelay = rouletteSelection(relays: closetsRelayGroup, weightFunction: { relay in
                UInt64(1 + greatestDistance - relay.distance)
            })

            return closestRelay?.relay
        }
    }

    private static func evaluatePort(
        relays: REST.ServerRelaysResponse,
        portConstraint: RelayConstraint<UInt16>,
        numberOfFailedAttempts: UInt
    ) throws -> UInt16 {
        let port = applyPortConstraint(
            portConstraint,
            rawPortRanges: relays.wireguard.portRanges,
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
        relays: REST.ServerRelaysResponse
    ) -> RelaySelectorMatch {
        let endpoint = MullvadEndpoint(
            ipv4Relay: IPv4Endpoint(
                ip: relayWithLocation.relay.ipv4AddrIn,
                port: port
            ),
            ipv6Relay: nil,
            ipv4Gateway: relays.wireguard.ipv4Gateway,
            ipv6Gateway: relays.wireguard.ipv6Gateway,
            publicKey: relayWithLocation.relay.publicKey
        )

        return RelaySelectorMatch(
            endpoint: endpoint,
            relay: relayWithLocation.relay,
            location: relayWithLocation.serverLocation
        )
    }
}
