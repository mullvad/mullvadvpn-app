//
//  SettingsManagerTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Testing

@testable import MullvadSettings
@testable import MullvadTypes

@Suite("SettingsManagerTests")
struct SettingsManagerTests {

    @Test(
        .serialized,
        arguments: [true, false, true])
    func enableRecentConnections(_ value: Bool) throws {
        let store = InMemorySettingsStore<SettingNotFound>()
        SettingsManager.unitTestStore = store
        try SettingsManager.enableRecentConnections(value)
        let storedValue = try SettingsManager.isRecentConnectionsShown()
        #expect(storedValue == value)
    }
}
