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
    public var filter: RelayConstraint<RelayFilter>

    public var debugDescription: String {
        "RelayConstraints { entry locations: \(entryLocations), exit locations: \(exitLocations) , port: \(port), filter: \(filter) }"
    }

    public init(
        entryLocations: RelayConstraint<UserSelectedRelays> = .only(UserSelectedRelays(locations: [.country("se")])),
        exitLocations: RelayConstraint<UserSelectedRelays> = .only(UserSelectedRelays(locations: [.country("se")])),
        port: RelayConstraint<UInt16> = .any,
        filter: RelayConstraint<RelayFilter> = .any
    ) {
        self.entryLocations = entryLocations
        self.exitLocations = exitLocations
        self.port = port
        self.filter = filter
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        // Added in 2023.3
        port = try container.decodeIfPresent(RelayConstraint<UInt16>.self, forKey: .port) ?? .any
        filter = try container.decodeIfPresent(RelayConstraint<RelayFilter>.self, forKey: .filter) ?? .any

        // Added in 2024.5
        entryLocations = try container.decodeIfPresent(
            RelayConstraint<UserSelectedRelays>.self,
            forKey: .entryLocations
        ) ?? .only(UserSelectedRelays(locations: [.country("se")]))

        exitLocations = try container
            .decodeIfPresent(RelayConstraint<UserSelectedRelays>.self, forKey: .exitLocations) ??
            container.decodeIfPresent(
                RelayConstraint<UserSelectedRelays>.self,
                forKey: .locations
            ) ??
            Self.migrateRelayLocation(decoder: decoder)
            ?? .only(UserSelectedRelays(locations: [.country("se")]))
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
