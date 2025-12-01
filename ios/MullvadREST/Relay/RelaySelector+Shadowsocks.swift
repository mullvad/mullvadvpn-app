//
//  RelaySelector+Shadowsocks.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-05-17.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension RelaySelector {
    public enum Shadowsocks {
        /**
         Returns random shadowsocks bridge config, otherwise `nil` if there are no shadowdsocks bridges.
         */
        public static func randomBridgeConfig(from relays: REST.ServerRelaysResponse) -> REST.ServerShadowsocks? {
            relays.bridge.shadowsocks.randomElement()
        }

        /// Returns the closest Shadowsocks relay using the given `location`, or a random relay if `constraints` were
        /// unsatisfiable.
        ///
        /// - Parameters:
        ///   - location: The user selected `location`
        ///   - relays: The list of relays to randomly select from.
        /// - Returns: A Shadowsocks relay or `nil` if no active relay were found.
        public static func closestBridge(
            location: RelayConstraint<UserSelectedRelays>,
            in relaysResponse: REST.ServerRelaysResponse
        ) -> REST.BridgeRelay? {
            // Bridges to select from.
            let mappedBridges = RelayWithLocation.locateRelays(
                relays: relaysResponse.bridge.relays,
                locations: relaysResponse.locations
            ).filter { bridge in
                bridge.relay.active
            }

            // Relays used to find the currently selected location.
            let mappedRelays = RelayWithLocation.locateRelays(
                relays: relaysResponse.wireguard.relays,
                locations: relaysResponse.locations
            )

            guard
                let selectedRelay = mappedRelays.first(where: { relay in
                    if let location = location.value?.locations.first {
                        relay.matches(location: location)
                    } else {
                        false
                    }
                })
            else {
                return mappedBridges.randomElement()?.relay
            }

            return RelaySelector.closestRelay(
                to: selectedRelay.serverLocation.geoCoordinate,
                using: mappedBridges
            ) as? REST.BridgeRelay
        }
    }
}
