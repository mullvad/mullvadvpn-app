//
//  TunnelSettingsManagerTests.swift
//  PacketTunnelCoreTests
//
//  Created by Mojgan on 2024-06-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadSettings
import MullvadTypes
import PacketTunnelCore
import XCTest

class TunnelSettingsManagerTests: XCTestCase {
    func testNotifyWhenSettingsLoaded() throws {
        var loadedConfiguration: Settings?
        let tunnelSettingsManager = TunnelSettingsManager(
            settingsReader: SettingsReaderStub.staticConfiguration(),
            onLoadSettingsHandler: { settings in
                loadedConfiguration = settings
            }
        )

        let mock = try XCTUnwrap(tunnelSettingsManager.read())
        XCTAssertEqual(loadedConfiguration, mock)
    }
}
