//
//  RecentSettings.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadTypes

public struct RecentSettings: Codable, Sendable, Equatable {
    let isEnabled: Bool
    let entryLocations: [UserSelectedRelays]
    let exitLocations: [UserSelectedRelays]
}
