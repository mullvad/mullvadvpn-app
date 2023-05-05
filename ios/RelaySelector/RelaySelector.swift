//
//  RelaySelector.swift
//  RelaySelector
//
//  Created by pronebird on 11/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

private let defaultPort: UInt16 = 53

public enum RelaySelector {
    /**
     Returns random shadowsocks TCP bridge, otherwise `nil` if there are no shadowdsocks bridges.
     */
    public static func getShadowsocksTCPBridge(relays: REST.ServerRelaysResponse) -> REST.ServerShadowsocks? {
        return relays.bridge.shadowsocks.filter { $0.protocol == "tcp" }.randomElement()
    }

    /**
     Filters relay list using given constraints and selects random relay.
     Throws an error if there are no relays satisfying the given constraints.
     */
    public static func evaluate(
        relays: REST.ServerRelaysResponse,
        constraints: RelayConstraints,
        numberOfFailedAttempts: UInt
    ) throws -> RelaySelectorResult {
        let filteredRelays = applyConstraints(constraints, relays: Self.parseRelaysResponse(relays))
        let port = applyConstraints(
            constraints,
            rawPortRanges: relays.wireguard.portRanges,
            numberOfFailedAttempts: numberOfFailedAttempts
        )

        guard let relayWithLocation = pickRandomRelay(relays: filteredRelays), let port = port else {
            throw NoRelaysSatisfyingConstraintsError()
        }

        let endpoint = MullvadEndpoint(
            ipv4Relay: IPv4Endpoint(
                ip: relayWithLocation.relay.ipv4AddrIn,
                port: port
            ),
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
    private static func applyConstraints(
        _ constraints: RelayConstraints,
        relays: [RelayWithLocation]
    ) -> [RelayWithLocation] {
        return relays.filter { relayWithLocation -> Bool in
            switch constraints.location {
            case .any:
                return true
            case let .only(relayConstraint):
                switch relayConstraint {
                case let .country(countryCode):
                    return relayWithLocation.location.countryCode == countryCode &&
                        relayWithLocation.relay.includeInCountry

                case let .city(countryCode, cityCode):
                    return relayWithLocation.location.countryCode == countryCode &&
                        relayWithLocation.location.cityCode == cityCode

                case let .hostname(countryCode, cityCode, hostname):
                    return relayWithLocation.location.countryCode == countryCode &&
                        relayWithLocation.location.cityCode == cityCode &&
                        relayWithLocation.relay.hostname == hostname
                }
            }
        }.filter { relayWithLocation -> Bool in
            return relayWithLocation.relay.active
        }
    }

    /// Produce a port that is either user provided or randomly selected, satisfying the given constraints.
    private static func applyConstraints(
        _ constraints: RelayConstraints,
        rawPortRanges: [[UInt16]],
        numberOfFailedAttempts: UInt
    ) -> UInt16? {
        // 1. First two attempts should pick a random port.
        // 2. The next two should pick port 53.
        // 3. Repeat steps 1 and 2.
        let useDefaultPort = (numberOfFailedAttempts % 4 == 2) || (numberOfFailedAttempts % 4 == 3)

        return useDefaultPort ? defaultPort : pickRandomPort(rawPortRanges: rawPortRanges)
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
        var i = (1 ... totalWeight).randomElement()!

        let randomRelay = relays.first { relayWithLocation -> Bool in
            let (result, isOverflow) = i
                .subtractingReportingOverflow(relayWithLocation.relay.weight)

            i = isOverflow ? 0 : result

            return i == 0
        }

        assert(randomRelay != nil, "At least one relay must've had a weight above 0")

        return randomRelay
    }

    private static func pickRandomPort(rawPortRanges: [[UInt16]]) -> UInt16? {
        let portRanges = parseRawPortRanges(rawPortRanges)
        let portAmount = portRanges.reduce(0) { partialResult, closedRange in
            return partialResult + closedRange.count
        }

        guard var portIndex = (0 ..< portAmount).randomElement() else {
            return nil
        }

        for range in portRanges {
            if portIndex < range.count {
                return UInt16(portIndex) + range.lowerBound
            } else {
                portIndex -= range.count
            }
        }

        assertionFailure("Port selection algorithm is broken!")

        return nil
    }

    private static func parseRawPortRanges(_ rawPortRanges: [[UInt16]]) -> [ClosedRange<UInt16>] {
        return rawPortRanges.compactMap { inputRange -> ClosedRange<UInt16>? in
            guard inputRange.count == 2 else { return nil }

            let startPort = inputRange[0]
            let endPort = inputRange[1]

            if startPort <= endPort {
                return startPort ... endPort
            } else {
                return nil
            }
        }
    }

    private static func parseRelaysResponse(_ response: REST.ServerRelaysResponse) -> [RelayWithLocation] {
        return response.wireguard.relays.compactMap { serverRelay -> RelayWithLocation? in
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

public struct NoRelaysSatisfyingConstraintsError: LocalizedError {
    public var errorDescription: String? {
        return "No relays satisfying constraints."
    }
}

public struct RelaySelectorResult: Codable {
    public var endpoint: MullvadEndpoint
    public var relay: REST.ServerRelay
    public var location: Location

    public var packetTunnelRelay: PacketTunnelRelay {
        return PacketTunnelRelay(
            ipv4Relay: endpoint.ipv4Relay,
            ipv6Relay: endpoint.ipv6Relay,
            hostname: relay.hostname,
            location: location
        )
    }
}

private struct RelayWithLocation {
    var relay: REST.ServerRelay
    var location: Location
}
