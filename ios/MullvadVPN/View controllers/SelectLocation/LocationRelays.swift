//
//  LocationRelays.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-08-12.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
