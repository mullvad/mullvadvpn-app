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
            preferClosest: Bool = false
        ) throws -> RelaySelectorMatch {
            let port = try evaluatePort(
                relays: relays,
                portConstraint: portConstraint,
                numberOfFailedAttempts: numberOfFailedAttempts
            )

            var relayWithLocation: RelayWithLocation<REST.ServerRelay>?
            if preferClosest {
                let relay = closestRelay(from: relayWithLocations)
                relayWithLocation = relayWithLocations.first(where: { $0.relay == relay })
            }

            guard
                let relayWithLocation = relayWithLocation ?? pickRandomRelayByWeight(relays: relayWithLocations)
            else {
                throw NoRelaysSatisfyingConstraintsError(.relayConstraintNotMatching)
            }

            return createMatch(for: relayWithLocation, port: port, relays: relays)
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
