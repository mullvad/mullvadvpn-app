//
//  RelaySelector.swift
//  RelaySelector
//
//  Created by pronebird on 11/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

private let defaultPort: UInt16 = 53

public enum RelaySelector {
    // MARK: - public

    /// Determines whether a `REST.ServerRelay` satisfies the given relay filter.
    public static func relayMatchesFilter(_ relay: AnyRelay, filter: RelayFilter) -> Bool {
        if case let .only(providers) = filter.providers, providers.contains(relay.provider) == false {
            return false
        }

        switch filter.ownership {
        case .any:
            return true
        case .owned:
            return relay.owned
        case .rented:
            return !relay.owned
        }
    }

    // MARK: - private

    static func pickRandomRelayByWeight<T: AnyRelay>(relays: [RelayWithLocation<T>])
        -> RelayWithLocation<T>? {
        rouletteSelection(relays: relays, weightFunction: { relayWithLocation in relayWithLocation.relay.weight })
    }

    private static func pickRandomPort(rawPortRanges: [[UInt16]]) -> UInt16? {
        let portRanges = parseRawPortRanges(rawPortRanges)
        let portAmount = portRanges.reduce(0) { partialResult, closedRange in
            partialResult + closedRange.count
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

    static func rouletteSelection<T>(relays: [T], weightFunction: (T) -> UInt64) -> T? {
        let totalWeight = relays.map { weightFunction($0) }.reduce(0) { accumulated, weight in
            accumulated + weight
        }
        // Return random relay when all relays within the list have zero weight.
        guard totalWeight > 0 else {
            return relays.randomElement()
        }

        // Pick a random number in the range 1 - totalWeight. This chooses the relay with a
        // non-zero weight.
        var i = (1 ... totalWeight).randomElement()!

        let randomRelay = relays.first { relay -> Bool in
            let (result, isOverflow) = i
                .subtractingReportingOverflow(weightFunction(relay))

            i = isOverflow ? 0 : result

            return i == 0
        }

        assert(randomRelay != nil, "At least one relay must've had a weight above 0")

        return randomRelay
    }

    static func mapRelays<T: AnyRelay>(
        relays: [T],
        locations: [String: REST.ServerLocation]
    ) -> [RelayWithLocation<T>] {
        relays.compactMap { relay in
            guard let serverLocation = locations[relay.location] else { return nil }
            return makeRelayWithLocationFrom(serverLocation, relay: relay)
        }
    }

    private static func makeRelayWithLocationFrom<T: AnyRelay>(
        _ serverLocation: REST.ServerLocation,
        relay: T
    ) -> RelayWithLocation<T>? {
        let locationComponents = relay.location.split(separator: "-")
        guard locationComponents.count > 1 else { return nil }

        let location = Location(
            country: serverLocation.country,
            countryCode: String(locationComponents[0]),
            city: serverLocation.city,
            cityCode: String(locationComponents[1]),
            latitude: serverLocation.latitude,
            longitude: serverLocation.longitude
        )

        return RelayWithLocation(relay: relay, serverLocation: location)
    }

    private static func parseRawPortRanges(_ rawPortRanges: [[UInt16]]) -> [ClosedRange<UInt16>] {
        rawPortRanges.compactMap { inputRange -> ClosedRange<UInt16>? in
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

    /// Produce a list of `RelayWithLocation` items satisfying the given constraints
    static func applyConstraints<T: AnyRelay>(
        _ relayConstraint: RelayConstraint<UserSelectedRelays>,
        filterConstraint: RelayConstraint<RelayFilter>,
        relays: [RelayWithLocation<T>]
    ) -> [RelayWithLocation<T>] {
        // Filter on active status, filter, and location.
        let filteredRelays = relays.filter { relayWithLocation -> Bool in
            guard relayWithLocation.relay.active else {
                return false
            }

            switch filterConstraint {
            case .any:
                break
            case let .only(filter):
                if !relayMatchesFilter(relayWithLocation.relay, filter: filter) {
                    return false
                }
            }

            return switch relayConstraint {
            case .any:
                true
            case let .only(relayConstraint):
                // At least one location must match the relay under test.
                relayConstraint.locations.contains { location in
                    relayWithLocation.matches(location: location)
                }
            }
        }

        // Filter on country inclusion.
        let includeInCountryFilteredRelays = filteredRelays.filter { relayWithLocation in
            return switch relayConstraint {
            case .any:
                true
            case let .only(relayConstraint):
                relayConstraint.locations.contains { location in
                    if case .country = location {
                        return relayWithLocation.relay.includeInCountry
                    }
                    return false
                }
            }
        }

        // If no relays should be included in the matched country, instead accept all.
        if includeInCountryFilteredRelays.isEmpty {
            return filteredRelays
        } else {
            return includeInCountryFilteredRelays
        }
    }

    /// Produce a port that is either user provided or randomly selected, satisfying the given constraints.
    static func applyPortConstraint(
        _ portConstraint: RelayConstraint<UInt16>,
        rawPortRanges: [[UInt16]],
        numberOfFailedAttempts: UInt
    ) -> UInt16? {
        switch portConstraint {
        case let .only(port):
            return port

        case .any:
            // 1. First two attempts should pick a random port.
            // 2. The next two should pick port 53.
            // 3. Repeat steps 1 and 2.
            let useDefaultPort = (numberOfFailedAttempts % 4 == 2) || (numberOfFailedAttempts % 4 == 3)

            return useDefaultPort ? defaultPort : pickRandomPort(rawPortRanges: rawPortRanges)
        }
    }
}
