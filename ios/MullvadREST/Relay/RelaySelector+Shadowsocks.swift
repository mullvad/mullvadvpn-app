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
        public static func closestBridge(
            location: RelayConstraint<UserSelectedRelays>,
            port: RelayConstraint<UInt16>,
            filter: RelayConstraint<RelayFilter>,
            in relaysResponse: REST.ServerRelaysResponse
        ) -> REST.BridgeRelay? {
            let mappedBridges = mapRelays(relays: relaysResponse.bridge.relays, locations: relaysResponse.locations)
            let filteredRelays = (try? applyConstraints(
                location,
                filterConstraint: filter,
                daitaEnabled: false,
                relays: mappedBridges
            )) ?? []

            guard filteredRelays.isEmpty == false else { return relay(from: relaysResponse) }

            return closestRelay(from: filteredRelays) ?? filteredRelays.randomElement()?.relay
        }
    }
}
