//
//  SettingsStore.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-11-22.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

// When adding new cases here, make sure to check if they should be added to the
// "partially" variable when reseting the store in SettingsManager.resetStore().
public enum SettingsKey: String, CaseIterable, Sendable, Codable {
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
