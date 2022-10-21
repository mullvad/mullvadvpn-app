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

    public var debugDescription: String {
        var output = "RelayConstraints { "
        output += "location: \(String(reflecting: location))"
        output += " }"
        return output
    }

    public init(location: RelayConstraint<RelayLocation> = .only(.country("se"))) {
        self.location = location
    }
}
