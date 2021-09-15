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
    var endpoint: MullvadEndpoint
    var relay: REST.ServerRelay
    var location: Location
}

private struct RelayWithLocation {
    var relay: REST.ServerRelay
    var location: Location
}

extension RelaySelectorResult {
    var tunnelConnectionInfo: TunnelConnectionInfo {
        return TunnelConnectionInfo(
            ipv4Relay: self.endpoint.ipv4Relay,
            ipv6Relay: self.endpoint.ipv6Relay,
            hostname: self.relay.hostname,
            location: self.location
        )
    }
}

struct RelaySelector {

    private let relays: REST.ServerRelaysResponse

    init(relays: REST.ServerRelaysResponse) {
        self.relays = relays
    }

    func evaluate(with constraints: RelayConstraints) -> RelaySelectorResult? {
        let filteredRelays = Self.applyConstraints(constraints, relays: Self.parseRelaysResponse(self.relays))
        let totalWeight = filteredRelays.reduce(0) { $0 + $1.relay.weight }

        guard totalWeight > 0 else { return nil }
        guard var i = (0...totalWeight).randomElement() else { return nil }

        let relayWithLocation = filteredRelays.first { (relayWithLocation) -> Bool in
            i -= relayWithLocation.relay.weight
            return i <= 0
        }.unsafelyUnwrapped

        guard let port = relays.wireguard.portRanges.randomElement()?.randomElement() else {
            return nil
        }

        let endpoint = MullvadEndpoint(
            ipv4Relay: IPv4Endpoint(ip: relayWithLocation.relay.ipv4AddrIn, port: port),
            ipv6Relay: nil,
            ipv4Gateway: relays.wireguard.ipv4Gateway,
            ipv6Gateway: relays.wireguard.ipv6Gateway,
            publicKey: relayWithLocation.relay.publicKey
        )

        return RelaySelectorResult(
            endpoint: endpoint,
            relay: relayWithLocation.relay,
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
        }.filter { (relayWithLocation) -> Bool in
            return relayWithLocation.relay.active
        }
    }

    private static func parseRelaysResponse(_ response: REST.ServerRelaysResponse) -> [RelayWithLocation] {
        return response.wireguard.relays.compactMap { (serverRelay) -> RelayWithLocation? in
            guard let serverLocation = response.locations[serverRelay.location] else { return nil }

            let locationComponents = serverRelay.location.split(separator: "-")
            guard locationComponents.count > 1 else { return nil }

            let location = Location(
                country: serverLocation.country,
                countryCode: String(locationComponents[0]),
                city: serverLocation.city,
                cityCode: String(locationComponents[1]),
                latitude: serverLocation.latitude,
                longitude: serverLocation.longitude
            )

            return RelayWithLocation(relay: serverRelay, location: location)
        }
    }

}
