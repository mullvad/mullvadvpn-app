//
//  RelayConstraint.swift
//  MullvadTypes
//
//  Created by pronebird on 10/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct RelayConstraints: Equatable, CustomDebugStringConvertible {
    public var location: RelayConstraint<RelayLocation> = .only(.country("se"))

    // Added in 2023.3
    public var port: RelayConstraint<UInt16> = .any

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
}

extension RelayConstraints: Codable {
    private enum CodingKeys: CodingKey {
        case location

        // Added in 2023.3
        case port
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        if let storedLocation = try container.decodeIfPresent(
            RelayConstraint<RelayLocation>.self,
            forKey: .location
        ) {
            location = storedLocation
        }

        // Added in 2023.3
        if let storedPort = try container.decodeIfPresent(
            RelayConstraint<UInt16>.self,
            forKey: .port
        ) {
            port = storedPort
        }
    }
}
