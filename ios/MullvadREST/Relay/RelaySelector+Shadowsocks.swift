// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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

            return RelaySelector.randomCloseRelay(
                to: selectedRelay.serverLocation.geoCoordinate,
                using: mappedBridges
            ) as? REST.BridgeRelay
        }
    }
}
