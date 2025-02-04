//
//  SettingsUpdaterTests.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2024-05-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

class SettingsUpdaterTests: XCTestCase {
    private var settingsListener: TunnelSettingsListener!
    private var settingsUpdater: SettingsUpdater!
    private var observers: [SettingsObserver]!

    override func setUp() {
        settingsListener = TunnelSettingsListener()
        settingsUpdater = SettingsUpdater(listener: settingsListener)
        observers = []
    }

    override func tearDown() {
        self.observers.forEach {
            settingsUpdater.removeObserver($0)
        }
    }

    func testSettingsListener() {
        var count = 0

        let latestSettings = LatestTunnelSettings()

        observers.append(SettingsObserverBlock(didUpdateSettings: { _ in
            count += 1
        }))

        observers.append(SettingsObserverBlock(didUpdateSettings: { _ in
            count += 1
        }))

        observers.forEach { settingsUpdater.addObserver($0) }

        settingsListener.onNewSettings?(latestSettings)

        XCTAssertEqual(count, 2)
    }
}
