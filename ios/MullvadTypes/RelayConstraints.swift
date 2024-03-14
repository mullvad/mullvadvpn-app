//
//  RelayConstraint.swift
//  MullvadTypes
//
//  Created by pronebird on 10/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol ConstraintsPropagation {
    var onNewConstraints: ((RelayConstraints) -> Void)? { get set }
}

public class RelayConstraintsUpdater: ConstraintsPropagation {
    public var onNewConstraints: ((RelayConstraints) -> Void)?

    public init(onNewConstraints: ((RelayConstraints) -> Void)? = nil) {
        self.onNewConstraints = onNewConstraints
    }
}

public struct RelayConstraints: Codable, Equatable, CustomDebugStringConvertible {
    @available(*, deprecated, renamed: "locations")
    private var location: RelayConstraint<RelayLocation> = .only(.country("se"))

    // Added in 2023.3
    public var port: RelayConstraint<UInt16>
    public var filter: RelayConstraint<RelayFilter>

    // Added in 2024.1
    // Changed from RelayLocations to UserSelectedRelays in 2024.3
    public var locations: RelayConstraint<UserSelectedRelays>

    public var debugDescription: String {
        "RelayConstraints { locations: \(locations), port: \(port), filter: \(filter) }"
    }

    public init(
        locations: RelayConstraint<UserSelectedRelays> = .only(UserSelectedRelays(locations: [.country("se")])),
        port: RelayConstraint<UInt16> = .any,
        filter: RelayConstraint<RelayFilter> = .any
    ) {
        self.locations = locations
        self.port = port
        self.filter = filter
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        // Added in 2023.3
        port = try container.decodeIfPresent(RelayConstraint<UInt16>.self, forKey: .port) ?? .any
        filter = try container.decodeIfPresent(RelayConstraint<RelayFilter>.self, forKey: .filter) ?? .any

        // Added in 2024.1
        locations = try container.decodeIfPresent(RelayConstraint<UserSelectedRelays>.self, forKey: .locations)
            ?? Self.migrateRelayLocations(decoder: decoder)
            ?? Self.migrateRelayLocation(decoder: decoder)
            ?? .only(UserSelectedRelays(locations: [.country("se")]))
    }
}

extension RelayConstraints {
    private static func migrateRelayLocations(decoder: Decoder) -> RelayConstraint<UserSelectedRelays>? {
        let container = try? decoder.container(keyedBy: CodingKeys.self)

        guard
            let relays = try? container?.decodeIfPresent(RelayConstraint<RelayLocations>.self, forKey: .locations)
        else {
            return nil
        }

        return switch relays {
        case .any:
            .any
        case let .only(relays):
            .only(UserSelectedRelays(
                locations: relays.locations,
                customListSelection: relays.customListId.flatMap { listId in
                    UserSelectedRelays.CustomListSelection(
                        listId: listId,
                        isList: false
                    )
                }
            ))
        }
    }

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
