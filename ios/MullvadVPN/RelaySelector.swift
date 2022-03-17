//
//  RelaySelector.swift
//  PacketTunnel
//
//  Created by pronebird on 11/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

struct RelaySelectorResult: Codable {
    var endpoint: MullvadEndpoint
    var relay: REST.ServerRelay
    var location: Location
}

private struct RelayWithLocation {
    var relay: REST.ServerRelay
    var location: Location
}

extension RelaySelectorResult {
    var packetTunnelRelay: PacketTunnelRelay {
        return PacketTunnelRelay(
            ipv4Relay: endpoint.ipv4Relay,
            ipv6Relay: endpoint.ipv6Relay,
            hostname: relay.hostname,
            location: location
        )
    }
}

enum RelaySelector {}

extension RelaySelector {

    static func evaluate(relays: REST.ServerRelaysResponse, constraints: RelayConstraints) -> RelaySelectorResult? {
        let filteredRelays = applyConstraints(constraints, relays: Self.parseRelaysResponse(relays))

        guard let relayWithLocation = pickRandomRelay(relays: filteredRelays) else {
            return nil
        }

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

    private static func pickRandomRelay(relays: [RelayWithLocation]) -> RelayWithLocation? {
        let totalWeight = relays.reduce(0) { accummulatedWeight, relayWithLocation in
            return accummulatedWeight + relayWithLocation.relay.weight
        }

        // Return random relay when all relays within the list have zero weight.
        guard totalWeight > 0 else {
            return relays.randomElement()
        }

        // Pick a random number in the range 1 - totalWeight. This choses the relay with a
        // non-zero weight.
        var i = (1...totalWeight).randomElement()!

        let randomRelay = relays.first { (relayWithLocation) -> Bool in
            let (result, isOverflow) = i.subtractingReportingOverflow(relayWithLocation.relay.weight)

            i = isOverflow ? 0 : result

            return i == 0
        }

        precondition(randomRelay != nil, "At least one relay must've had a weight above 0")

        return randomRelay
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
