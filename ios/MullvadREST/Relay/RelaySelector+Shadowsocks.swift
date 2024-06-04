//
//  RelaySelector+Shadowsocks.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-05-17.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension RelaySelector {
    public enum Shadowsocks {
        /**
         Returns random shadowsocks TCP bridge, otherwise `nil` if there are no shadowdsocks bridges.
         */
        public static func tcpBridge(from relays: REST.ServerRelaysResponse) -> REST.ServerShadowsocks? {
            relays.bridge.shadowsocks.filter { $0.protocol == "tcp" }.randomElement()
        }

        /// Return a random Shadowsocks bridge relay, or `nil` if no relay were found.
        ///
        /// Non `active` relays are filtered out.
        /// - Parameter relays: The list of relays to randomly select from.
        /// - Returns: A Shadowsocks relay or `nil` if no active relay were found.
        public static func relay(from relaysResponse: REST.ServerRelaysResponse) -> REST.BridgeRelay? {
            relaysResponse.bridge.relays.filter { $0.active }.randomElement()
        }

        /// Returns the closest Shadowsocks relay using the given `location`, or a random relay if `constraints` were
        /// unsatisfiable.
        ///
        /// - Parameters:
        ///   - location: The user selected `location`
        ///   - port: The user selected port
        ///   - filter: The user filtered criteria
        ///   - relays: The list of relays to randomly select from.
        /// - Returns: A Shadowsocks relay or `nil` if no active relay were found.
        public static func closestRelay(
            location: RelayConstraint<UserSelectedRelays>,
            port: RelayConstraint<UInt16>,
            filter: RelayConstraint<RelayFilter>,
            in relaysResponse: REST.ServerRelaysResponse
        ) -> REST.BridgeRelay? {
            let mappedBridges = mapRelays(relays: relaysResponse.bridge.relays, locations: relaysResponse.locations)
            let filteredRelays = applyConstraints(
                location,
                filterConstraint: filter,
                relays: mappedBridges
            )
            guard filteredRelays.isEmpty == false else { return relay(from: relaysResponse) }

            // Compute the midpoint location from all the filtered relays
            // Take *either* the first five relays, OR the relays below maximum bridge distance
            // sort all of them by Haversine distance from the computed midpoint location
            // then use the roulette selection to pick a bridge

            let midpointDistance = Midpoint.location(in: filteredRelays.map { $0.serverLocation.geoCoordinate })
            let maximumBridgeDistance = 1500.0
            let relaysWithDistance = filteredRelays.map {
                RelayWithDistance(
                    relay: $0.relay,
                    distance: Haversine.distance(
                        midpointDistance.latitude,
                        midpointDistance.longitude,
                        $0.serverLocation.latitude,
                        $0.serverLocation.longitude
                    )
                )
            }.sorted {
                $0.distance < $1.distance
            }.filter {
                $0.distance <= maximumBridgeDistance
            }.prefix(5)

            var greatestDistance = 0.0
            relaysWithDistance.forEach {
                if $0.distance > greatestDistance {
                    greatestDistance = $0.distance
                }
            }

            let randomRelay = rouletteSelection(relays: Array(relaysWithDistance), weightFunction: { relay in
                UInt64(1 + greatestDistance - relay.distance)
            })

            return randomRelay?.relay ?? filteredRelays.randomElement()?.relay
        }
    }
}
