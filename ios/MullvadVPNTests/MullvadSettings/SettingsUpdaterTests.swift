import XCTest

// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only
@testable import MullvadSettings
@testable import MullvadTypes

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

        observers.append(
            SettingsObserverBlock(didUpdateSettings: { _ in
                count += 1
            }))

        observers.append(
            SettingsObserverBlock(didUpdateSettings: { _ in
                count += 1
            }))

        observers.forEach { settingsUpdater.addObserver($0) }

        settingsListener.onNewSettings?(latestSettings)

        XCTAssertEqual(count, 2)
    }
}
