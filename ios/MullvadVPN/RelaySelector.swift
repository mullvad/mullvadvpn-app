//
//  RelaySelector.swift
//  PacketTunnel
//
//  Created by pronebird on 11/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

struct RelaySelectorResult {
    var relay: RelayList.Hostname
    var tunnel: RelayList.WireguardTunnel
    var endpoint: MullvadEndpoint
    var geoLocation: GeoLocation
}

struct RelaySelector {

    private let relayList: RelayList

    init(relayList: RelayList) {
        self.relayList = relayList
    }

    /// Produce a `RelayList` satisfying the given constraints
    private func applyConstraints(_ constraints: RelayConstraints) -> RelayList {
        let filteredCountries = relayList.countries.filter { (country) -> Bool in
            switch constraints.location {
            case .any:
                return true
            case .only(let constraint):
                switch constraint {
                case .country(let countryCode):
                    return countryCode == country.code
                case .city(let countryCode, _):
                    return countryCode == country.code
                case .hostname(let countryCode, _, _):
                    return countryCode == country.code
                }
            }
        }.map { (country) -> RelayList.Country in
            var filteredCountry = country
            filteredCountry.cities = country.cities.filter { (city) -> Bool in
                switch constraints.location {
                case .any:
                    return true
                case .only(let constraint):
                    switch constraint {
                    case .country:
                        return true
                    case .city(_, let cityCode):
                        return cityCode == city.code
                    case .hostname(_, let cityCode, _):
                        return cityCode == city.code
                    }
                }
            }.map { (city) -> RelayList.City in
                var filteredCity = city
                filteredCity.relays = city.relays.filter { (relay) -> Bool in
                    switch constraints.location {
                    case .any:
                        return true
                    case .only(let constraint):
                        switch constraint {
                        case .country, .city:
                            return true
                        case .hostname(_, _, let hostname):
                            return hostname == relay.hostname
                        }
                    }
                }
                .map({ (relay) -> RelayList.Hostname in
                    var filteredRelay = relay
                    filteredRelay.tunnels?.wireguard = relay.tunnels?.wireguard?
                        .filter { !$0.portRanges.isEmpty }

                    return filteredRelay
                }).filter { (relay) -> Bool in
                    guard let wireguardTunnels = relay.tunnels?.wireguard else { return false }

                    return relay.active && !wireguardTunnels.isEmpty
                }

                return filteredCity
            }.filter({ (city) -> Bool in
                return !city.relays.isEmpty
            })

            return filteredCountry
        }.filter { (country) -> Bool in
            return !country.cities.isEmpty
        }

        return RelayList(countries: filteredCountries)
    }

    func evaluate(with constraints: RelayConstraints) -> RelaySelectorResult? {
        let filteredRelayList = applyConstraints(constraints)

        guard let country = filteredRelayList.countries.randomElement() else {
            return nil
        }

        guard let city = country.cities.randomElement() else {
            return nil
        }

        guard let relay = city.relays.randomElement() else {
                return nil
        }

        guard let tunnel = relay.tunnels?.wireguard?.randomElement() else {
            return nil
        }

        guard let port = tunnel.portRanges.randomElement()?.randomElement() else {
            return nil
        }

        let endpoint = MullvadEndpoint(
            ipv4Relay: IPv4Endpoint(ip: relay.ipv4AddrIn, port: port),
            ipv6Relay: nil,
            ipv4Gateway: tunnel.ipv4Gateway,
            ipv6Gateway: tunnel.ipv6Gateway,
            publicKey: tunnel.publicKey
        )

        let geoLocation = GeoLocation(
            country: country.name,
            city: city.name,
            latitude: city.latitude,
            longitude: city.longitude
        )

        return RelaySelectorResult(
            relay: relay,
            tunnel: tunnel,
            endpoint: endpoint,
            geoLocation: geoLocation
        )
    }

}
