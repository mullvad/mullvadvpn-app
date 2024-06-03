//
//  RelaySelector+Wireguard.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-05-17.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension RelaySelector {
    public enum WireGuard {
        /**
         Filters relay list using given constraints and selects random relay for exit relay.
         Throws an error if there are no relays satisfying the given constraints.
         */
        public static func evaluate(
            by constraints: RelayConstraints,
            in relaysResponse: REST.ServerRelaysResponse,
            numberOfFailedAttempts: UInt
        ) throws -> RelaySelectorResult {
            let exitCandidates = try findBestMatch(
                relays: relaysResponse,
                relayConstraint: constraints.exitLocations,
                portConstraint: constraints.port,
                filterConstraint: constraints.filter,
                numberOfFailedAttempts: numberOfFailedAttempts
            )

            return exitCandidates
        }

        // MARK: - private functions

        private static func findBestMatch(
            relays: REST.ServerRelaysResponse,
            relayConstraint: RelayConstraint<UserSelectedRelays>,
            portConstraint: RelayConstraint<UInt16>,
            filterConstraint: RelayConstraint<RelayFilter>,
            numberOfFailedAttempts: UInt
        ) throws -> RelaySelectorMatch {
            let mappedRelays = mapRelays(relays: relays.wireguard.relays, locations: relays.locations)
            let filteredRelays = applyConstraints(
                relayConstraint,
                portConstraint: portConstraint,
                filterConstraint: filterConstraint,
                relays: mappedRelays
            )
            let port = applyPortConstraint(
                portConstraint,
                rawPortRanges: relays.wireguard.portRanges,
                numberOfFailedAttempts: numberOfFailedAttempts
            )

            guard let relayWithLocation = pickRandomRelayByWeight(relays: filteredRelays), let port else {
                throw NoRelaysSatisfyingConstraintsError()
            }

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
}
