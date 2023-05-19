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
}
