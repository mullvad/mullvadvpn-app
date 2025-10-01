//
//  RecentConnectionLocationNode.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-01.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadTypes
class RecentConnectionLocationNode: RecentsLocationNodeRepresentable, @unchecked Sendable {
    var name: String
    var code: String
    var isActive: Bool
    var entryLocation: LocationNode?
    var exitLocation: LocationNode
    init(name: String, code: String, isActive: Bool, entryLocation: LocationNode?, exitLocation: LocationNode) {
        self.name = name
        self.code = code
        self.isActive = isActive
        self.entryLocation = entryLocation
        self.exitLocation = exitLocation
    }
}
