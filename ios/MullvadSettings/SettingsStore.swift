//
//  SettingsStore.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-11-22.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum SettingsKey: String, CaseIterable {
    case settings = "Settings"
    case deviceState = "DeviceState"
    case apiAccessMethods = "ApiAccessMethods"
    case ipOverrides = "IPOverrides"
    case lastUsedAccount = "LastUsedAccount"
    case shouldWipeSettings = "ShouldWipeSettings"
}

public protocol SettingsStore {
    func read(key: SettingsKey) throws -> Data
    func write(_ data: Data, for key: SettingsKey) throws
    func delete(key: SettingsKey) throws
}
