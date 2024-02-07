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
    public var locations: RelayConstraint<RelayLocations>

    // Added in 2023.3
    public var port: RelayConstraint<UInt16>
    public var filter: RelayConstraint<RelayFilter>

    public var debugDescription: String {
        "RelayConstraints { locations: \(locations), port: \(port) }"
    }

    public init(
        locations: RelayConstraint<RelayLocations> = .only(RelayLocations(locations: [.country("se")])),
        port: RelayConstraint<UInt16> = .any,
        filter: RelayConstraint<RelayFilter> = .any
    ) {
        self.locations = locations
        self.port = port
        self.filter = filter
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        locations = try container.decode(RelayConstraint<RelayLocations>.self, forKey: .locations)

        // Added in 2023.3
        port = try container.decodeIfPresent(RelayConstraint<UInt16>.self, forKey: .port) ?? .any
        filter = try container.decodeIfPresent(RelayConstraint<RelayFilter>.self, forKey: .filter) ?? .any
    }
}
