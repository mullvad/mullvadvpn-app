//
//  RelayConstraint.swift
//  MullvadTypes
//
//  Created by pronebird on 10/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct RelayConstraints: Codable, Equatable, CustomDebugStringConvertible {
    public var location: RelayConstraint<RelayLocation>

    // Added in 2023.3
    public var port: RelayConstraint<UInt16>

    public var debugDescription: String {
        return "RelayConstraints { location: \(location), port: \(port) }"
    }

    public init(
        location: RelayConstraint<RelayLocation> = .only(.country("se")),
        port: RelayConstraint<UInt16> = .any
    ) {
        self.location = location
        self.port = port
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        location = try container.decode(RelayConstraint<RelayLocation>.self, forKey: .location)

        // Added in 2023.3
        port = try container.decodeIfPresent(RelayConstraint<UInt16>.self, forKey: .port) ?? .any
    }
}
