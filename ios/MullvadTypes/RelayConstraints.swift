//
//  RelayConstraint.swift
//  MullvadTypes
//
//  Created by pronebird on 10/06/2019.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct RelayConstraints: Codable, Equatable, CustomDebugStringConvertible, @unchecked Sendable {
    @available(*, deprecated, renamed: "locations")
    private var location: RelayConstraint<RelayLocation> = .only(.country("se"))

    // Added in 2024.1
    // Changed from RelayLocations to UserSelectedRelays in 2024.3
    @available(*, deprecated, renamed: "exitLocations")
    private var locations: RelayConstraint<UserSelectedRelays> = .only(UserSelectedRelays(locations: [.country("se")]))

    // Added in 2024.5 to support multi-hop
    public var entryLocations: RelayConstraint<UserSelectedRelays>
    public var exitLocations: RelayConstraint<UserSelectedRelays>

    // Added in 2023.3
    public var port: RelayConstraint<UInt16>
    @available(*, deprecated, renamed: "entryFilter_exitFilter")
    public var filter: RelayConstraint<RelayFilter> = .any

    // Added in 2025.9
    public var entryFilter: RelayConstraint<RelayFilter>
    public var exitFilter: RelayConstraint<RelayFilter>

    public var debugDescription: String {
        "RelayConstraints { entryLocations: \(entryLocations), exitLocations: \(exitLocations), port: \(port), entryFilter: \(entryFilter), exitFilter: \(entryFilter) }"
    }

    public init(
        entryLocations: RelayConstraint<UserSelectedRelays> = .only(UserSelectedRelays(locations: [.country("se")])),
        exitLocations: RelayConstraint<UserSelectedRelays> = .only(UserSelectedRelays(locations: [.country("se")])),
        port: RelayConstraint<UInt16> = .any,
        entryFilter: RelayConstraint<RelayFilter> = .any,
        exitFilter: RelayConstraint<RelayFilter> = .any
    ) {
        self.entryLocations = entryLocations
        self.exitLocations = exitLocations
        self.port = port
        self.entryFilter = entryFilter
        self.exitFilter = exitFilter
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        // Added in 2023.3
        port = try container.decodeIfPresent(RelayConstraint<UInt16>.self, forKey: .port) ?? .any

        // Added in 2024.5
        entryLocations =
            try container.decodeIfPresent(
                RelayConstraint<UserSelectedRelays>.self,
                forKey: .entryLocations
            ) ?? .only(UserSelectedRelays(locations: [.country("se")]))

        exitLocations =
            try container
            .decodeIfPresent(RelayConstraint<UserSelectedRelays>.self, forKey: .exitLocations)
            ?? container.decodeIfPresent(
                RelayConstraint<UserSelectedRelays>.self,
                forKey: .locations
            ) ?? Self.migrateRelayLocation(decoder: decoder)
            ?? .only(UserSelectedRelays(locations: [.country("se")]))

        // Added in 2025.9
        entryFilter =
            try container.decodeIfPresent(RelayConstraint<RelayFilter>.self, forKey: .entryFilter)
            ?? container.decodeIfPresent(RelayConstraint<RelayFilter>.self, forKey: .filter)
            ?? .any

        exitFilter =
            try container.decodeIfPresent(RelayConstraint<RelayFilter>.self, forKey: .exitFilter)
            ?? container.decodeIfPresent(RelayConstraint<RelayFilter>.self, forKey: .filter)
            ?? .any
    }
}

extension RelayConstraints {
    private static func migrateRelayLocation(decoder: Decoder) -> RelayConstraint<UserSelectedRelays>? {
        let container = try? decoder.container(keyedBy: CodingKeys.self)

        guard
            let relay = try? container?.decodeIfPresent(RelayConstraint<RelayLocation>.self, forKey: .location)
        else {
            return nil
        }

        return switch relay {
        case .any:
            .any
        case let .only(relay):
            .only(UserSelectedRelays(locations: [relay]))
        }
    }
}

extension RelayConstraints {
    public func filterConstraint(for multihopContext: MultihopContext) -> RelayConstraint<RelayFilter> {
        switch multihopContext {
        case .entry:
            entryFilter
        case .exit:
            exitFilter
        }
    }

    public mutating func setFilterConstraint(
        _ constraint: RelayConstraint<RelayFilter>,
        for multihopContext: MultihopContext
    ) {
        switch multihopContext {
        case .entry:
            entryFilter = constraint
        case .exit:
            exitFilter = constraint
        }

        // Unique entry and exit filters are not ready yet for production use. A reminder to remove
        // this debug condition has been added to:
        // https://linear.app/mullvad/issue/IOS-1357/use-unique-entry-and-exit-filters-in-select-location-and-filter-view
        #if !DEBUG
            entryFilter = constraint
            exitFilter = constraint
        #endif
    }
}
