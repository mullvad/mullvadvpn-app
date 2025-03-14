//
//  RelayWithLocation.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-05-17.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public struct RelayWithLocation<T: AnyRelay & Sendable>: Sendable {
    public let relay: T
    public let serverLocation: Location

    public func matches(location: RelayLocation) -> Bool {
        return switch location {
        case let .country(countryCode):
            serverLocation.countryCode == countryCode

        case let .city(countryCode, cityCode):
            serverLocation.countryCode == countryCode &&
                serverLocation.cityCode == cityCode

        case let .hostname(countryCode, cityCode, hostname):
            serverLocation.countryCode == countryCode &&
                serverLocation.cityCode == cityCode &&
                relay.hostname == hostname
        }
    }

    init(relay: T, serverLocation: Location) {
        self.relay = relay
        self.serverLocation = serverLocation
    }

    init?(_ relay: T, locations: [String: REST.ServerLocation]) {
        guard
            let serverLocation = locations[relay.location.rawValue]
        else { return nil }

        self.relay = relay
        self.serverLocation = Location(
            country: serverLocation.country,
            countryCode: String(relay.location.country),
            city: serverLocation.city,
            cityCode: String(relay.location.city),
            latitude: serverLocation.latitude,
            longitude: serverLocation.longitude
        )
    }

    /// given a list of `AnyRelay` values and a name to location mapping, produce a list of
    /// `RelayWithLocation`values  for those whose locations have successfully been found.
    public static func locateRelays(
        relays: [T],
        locations: [String: REST.ServerLocation]
    ) -> [RelayWithLocation<T>] {
        relays.compactMap { RelayWithLocation($0, locations: locations) }
    }
}

extension RelayWithLocation: Hashable {
    public static func == (lhs: RelayWithLocation<T>, rhs: RelayWithLocation<T>) -> Bool {
        lhs.relay.hostname == rhs.relay.hostname
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(relay.hostname)
    }
}
