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
    public var locations: RelayConstraint<UserSelectedRelays>

    public var debugDescription: String {
        "RelayConstraints { locations: \(locations), port: \(port) }"
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
            ?? Self.migrateLocations(decoder: decoder)
            ?? .only(UserSelectedRelays(locations: [.country("se")]))
    }
}

extension RelayConstraints {
    private static func migrateLocations(decoder: Decoder) -> RelayConstraint<UserSelectedRelays>? {
        let container = try? decoder.container(keyedBy: CodingKeys.self)

        guard
            let location = try? container?.decodeIfPresent(RelayConstraint<RelayLocation>.self, forKey: .location)
        else {
            return nil
        }

        switch location {
        case .any:
            return .any
        case let .only(location):
            return .only(UserSelectedRelays(locations: [location]))
        }
    }
}
