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
    var relay: RelayList.Relay
    var tunnel: RelayList.WireguardTunnel
    var endpoint: MullvadEndpoint
    var location: Location
}

private struct RelayWithLocation {
    var relay: RelayList.Relay
    var location: Location
}

struct RelaySelector {

    private let relayList: RelayList

    init(relayList: RelayList) {
        self.relayList = relayList
    }

    func evaluate(with constraints: RelayConstraints) -> RelaySelectorResult? {
        let relays = Self.applyConstraints(constraints, relays: Self.parseRelayList(self.relayList))
        let totalWeight = relays.reduce(0) { $0 + $1.relay.weight }

        guard totalWeight > 0 else { return nil }
        guard var i = (0...totalWeight).randomElement() else { return nil }

        let relayWithLocation = relays.first { (relayWithLocation) -> Bool in
            i -= relayWithLocation.relay.weight
            return i <= 0
        }.unsafelyUnwrapped

        guard let tunnel = relayWithLocation.relay.tunnels?.wireguard?.randomElement() else {
            return nil
        }

        guard let port = tunnel.portRanges.randomElement()?.randomElement() else {
            return nil
        }

        let endpoint = MullvadEndpoint(
            ipv4Relay: IPv4Endpoint(ip: relayWithLocation.relay.ipv4AddrIn, port: port),
            ipv6Relay: nil,
            ipv4Gateway: tunnel.ipv4Gateway,
            ipv6Gateway: tunnel.ipv6Gateway,
            publicKey: tunnel.publicKey
        )

        return RelaySelectorResult(
            relay: relayWithLocation.relay,
            tunnel: tunnel,
            endpoint: endpoint,
            location: relayWithLocation.location
        )
    }

    /// Produce a list of `RelayWithLocation` items satisfying the given constraints
    private static func applyConstraints(_ constraints: RelayConstraints, relays: [RelayWithLocation]) -> [RelayWithLocation] {
        return relays.filter { (relayWithLocation) -> Bool in
            switch constraints.location {
            case .any:
                return true
            case .only(let relayConstraint):
                switch relayConstraint {
                case .country(let countryCode):
                    return relayWithLocation.location.countryCode == countryCode &&
                        relayWithLocation.relay.includeInCountry

                case .city(let countryCode, let cityCode):
                    return relayWithLocation.location.countryCode == countryCode &&
                        relayWithLocation.location.cityCode == cityCode

                case .hostname(let countryCode, let cityCode, let hostname):
                    return relayWithLocation.location.countryCode == countryCode &&
                        relayWithLocation.location.cityCode == cityCode &&
                        relayWithLocation.relay.hostname == hostname
                }
            }
        }.map({ (relayWithLocation) -> RelayWithLocation in
            var filteredRelay = relayWithLocation
            let wireguardTunnels = filteredRelay.relay.tunnels?.wireguard?
                .filter { !$0.portRanges.isEmpty }

            filteredRelay.relay.tunnels?.wireguard = wireguardTunnels

            return filteredRelay
        }).filter { (relayWithLocation) -> Bool in
            guard let wireguardTunnels = relayWithLocation.relay.tunnels?.wireguard else { return false }

            return relayWithLocation.relay.active && !wireguardTunnels.isEmpty
        }
    }

    private static func parseRelayList(_ relayList: RelayList) -> [RelayWithLocation] {
        var relays = [RelayWithLocation]()

        for country in relayList.countries {
            for city in country.cities {
                for relay in city.relays {
                    let location = Location(
                        country: country.name,
                        countryCode: country.code,
                        city: city.name,
                        cityCode: city.code,
                        latitude: city.latitude,
                        longitude: city.longitude
                    )
                    let relayWithLocation = RelayWithLocation(
                        relay: relay,
                        location: location
                    )
                    relays.append(relayWithLocation)
                }
            }
        }

        return relays
    }

}
