
//
//  RelayList.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import Network

struct RelayList: Codable {
    struct Country: Codable {
        var name: String
        var code: String
        var cities: [City]
    }

    struct City: Codable {
        var name: String
        var code: String
        var latitude: Double
        var longitude: Double
        var relays: [Hostname]
    }

    struct Hostname: Codable {
        var hostname: String
        var ipv4AddrIn: IPv4Address
        var includeInCountry: Bool
        var active: Bool
        var weight: Int32
        var tunnels: Tunnels?
    }

    struct Tunnels: Codable {
        var wireguard: [WireguardTunnel]?
    }

    struct WireguardTunnel: Codable {
        var ipv4Gateway: IPv4Address
        var ipv6Gateway: IPv6Address
        var publicKey: Data
        var portRanges: [ClosedRange<UInt16>]
    }

    var countries: [Country]
}

extension RelayList {

    /// Returns an alphabetically sorted `RelayList`
    func sorted() -> Self {
        let comparator = { (a: String, b: String) -> Bool in
            return a.localizedCaseInsensitiveCompare(b) == .orderedAscending
        }

        let sortedCountries = countries
            .sorted { comparator($0.name, $1.name) }
            .map { (country) -> RelayList.Country in
                var sortedCountry = country
                sortedCountry.cities = country.cities.sorted { comparator($0.name, $1.name) }
                    .map({ (city) -> RelayList.City in
                        var sortedCity = city
                        sortedCity.relays = city.relays
                            .sorted { comparator($0.hostname, $1.hostname) }
                        return sortedCity
                    })
                return sortedCountry
        }

        return RelayList(countries: sortedCountries)
    }

}
