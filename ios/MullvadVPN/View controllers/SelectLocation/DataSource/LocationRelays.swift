// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadREST

struct LocationRelays: Sendable {
    var relays: [REST.ServerRelay]
    var locations: [String: REST.ServerLocation]
}

extension Array where Element == RelayWithLocation<REST.ServerRelay> {
    func toLocationRelays() -> LocationRelays {
        return LocationRelays(
            relays: map { $0.relay },
            locations: reduce(into: [String: REST.ServerLocation]()) { result, entry in
                result[entry.relay.location.rawValue] = REST.ServerLocation(
                    country: entry.serverLocation.country,
                    city: entry.serverLocation.city,
                    latitude: entry.serverLocation.latitude,
                    longitude: entry.serverLocation.longitude
                )
            }
        )
    }
}
