//
//  RelaySelector.swift
//  RelaySelector
//
//  Created by pronebird on 11/06/2019.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

private let defaultPort: UInt16 = 443

public enum RelaySelector {
    // MARK: - public

    /// Determines whether a `REST.ServerRelay` satisfies the given relay filter.
    static func relayMatchesFilter(_ relay: AnyRelay, filter: RelayFilter) -> Bool {
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

    static func pickRandomRelayByWeight<T: AnyRelay>(relays: [RelayWithLocation<T>])
        -> RelayWithLocation<T>? {
        rouletteSelection(relays: relays, weightFunction: { relayWithLocation in relayWithLocation.relay.weight })
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

    /// Produce a list of `RelayWithLocation` items satisfying the given constraints
    static func applyConstraints<T: AnyRelay>(
        _ relayConstraint: RelayConstraint<UserSelectedRelays>,
        filterConstraint: RelayConstraint<RelayFilter>,
        daitaEnabled: Bool,
        relays: [RelayWithLocation<T>]
    ) throws -> [RelayWithLocation<T>] {
        // Filter on active status, daita support, filter constraint and relay constraint.
        var filteredRelays = try filterByActive(relays: relays)
        filteredRelays = try filterByFilterConstraint(relays: filteredRelays, constraint: filterConstraint)
        filteredRelays = try filterByLocationConstraint(relays: filteredRelays, constraint: relayConstraint)
        filteredRelays = try filterByDaita(relays: filteredRelays, daitaEnabled: daitaEnabled)
        return filterByCountryInclusion(relays: filteredRelays, constraint: relayConstraint)
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
            // 1. First attempt should pick a random port.
            // 2. The second should pick port 443.
            // 3. Repeat steps 1 and 2.
            let useDefaultPort = numberOfFailedAttempts.isOrdered(nth: 2, forEverySetOf: 2)

            return useDefaultPort ? defaultPort : pickRandomPort(rawPortRanges: rawPortRanges)
        }
    }

    // MARK: - private

    static func parseRawPortRanges(_ rawPortRanges: [[UInt16]]) -> [ClosedRange<UInt16>] {
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

    static func pickRandomPort(rawPortRanges: [[UInt16]]) -> UInt16? {
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

    private static func makeRelayWithLocationFrom<T: AnyRelay>(
        _ serverLocation: REST.ServerLocation,
        relay: T
    ) -> RelayWithLocation<T>? {
        let location = Location(
            country: serverLocation.country,
            countryCode: String(relay.location.country),
            city: serverLocation.city,
            cityCode: String(relay.location.city),
            latitude: serverLocation.latitude,
            longitude: serverLocation.longitude
        )

        return RelayWithLocation(relay: relay, serverLocation: location)
    }

    private static func filterByActive<T: AnyRelay>(
        relays: [RelayWithLocation<T>]
    ) throws -> [RelayWithLocation<T>] {
        let filteredRelays = relays.filter { relayWithLocation in
            relayWithLocation.relay.active
        }

        return if filteredRelays.isEmpty {
            throw NoRelaysSatisfyingConstraintsError(.noActiveRelaysFound)
        } else {
            filteredRelays
        }
    }

    private static func filterByDaita<T: AnyRelay>(
        relays: [RelayWithLocation<T>],
        daitaEnabled: Bool
    ) throws -> [RelayWithLocation<T>] {
        guard daitaEnabled else { return relays }

        let filteredRelays = relays.filter { relayWithLocation in
            relayWithLocation.relay.daita == true
        }

        return if filteredRelays.isEmpty {
            throw NoRelaysSatisfyingConstraintsError(.noDaitaRelaysFound)
        } else {
            filteredRelays
        }
    }

    private static func filterByFilterConstraint<T: AnyRelay>(
        relays: [RelayWithLocation<T>],
        constraint: RelayConstraint<RelayFilter>
    ) throws -> [RelayWithLocation<T>] {
        let filteredRelays = relays.filter { relayWithLocation in
            switch constraint {
            case .any:
                true
            case let .only(filter):
                relayMatchesFilter(relayWithLocation.relay, filter: filter)
            }
        }

        return if filteredRelays.isEmpty {
            throw NoRelaysSatisfyingConstraintsError(.filterConstraintNotMatching)
        } else {
            filteredRelays
        }
    }

    private static func filterByLocationConstraint<T: AnyRelay>(
        relays: [RelayWithLocation<T>],
        constraint: RelayConstraint<UserSelectedRelays>
    ) throws -> [RelayWithLocation<T>] {
        let filteredRelays = relays.filter { relayWithLocation in
            switch constraint {
            case .any:
                true
            case let .only(constraint):
                // At least one location must match the relay under test.
                constraint.locations.contains { location in
                    relayWithLocation.matches(location: location)
                }
            }
        }

        return if filteredRelays.isEmpty {
            throw NoRelaysSatisfyingConstraintsError(.relayConstraintNotMatching)
        } else {
            filteredRelays
        }
    }

    private static func filterByCountryInclusion<T: AnyRelay>(
        relays: [RelayWithLocation<T>],
        constraint: RelayConstraint<UserSelectedRelays>
    ) -> [RelayWithLocation<T>] {
        let filteredRelays = relays.filter { relayWithLocation in
            return switch constraint {
            case .any:
                true
            case let .only(relayConstraint):
                relayConstraint.locations.contains { location in
                    if case .country = location {
                        relayWithLocation.relay.includeInCountry
                    } else {
                        false
                    }
                }
            }
        }

        // If no relays are included in the matched country, instead accept all.
        return if filteredRelays.isEmpty {
            relays
        } else {
            filteredRelays
        }
    }
}
