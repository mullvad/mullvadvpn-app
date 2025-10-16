//
//  SettingsStore.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-11-22.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum SettingsKey: String, CaseIterable, Sendable {
    case settings = "Settings"
    case deviceState = "DeviceState"
    case apiAccessMethods = "ApiAccessMethods"
    case ipOverrides = "IPOverrides"
    case customRelayLists = "CustomRelayLists"
    case lastUsedAccount = "LastUsedAccount"
    case shouldWipeSettings = "ShouldWipeSettings"
    case recentConnections = "RecentConnections"
}

public protocol SettingsStore: Sendable {
    func read(key: SettingsKey) throws -> Data
    func write(_ data: Data, for key: SettingsKey) throws
    func delete(key: SettingsKey) throws
}
