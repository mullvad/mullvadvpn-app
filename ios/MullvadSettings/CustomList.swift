//
//  CustomList.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-01-25.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public struct CustomList: Codable, Equatable {
    public let id: UUID
    public var name: String
    public var locations: [RelayLocation]

    public init(id: UUID = UUID(), name: String, locations: [RelayLocation]) {
        self.id = id
        self.name = name
        self.locations = locations
    }
}
