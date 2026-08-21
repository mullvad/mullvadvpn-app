// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadTypes
import PacketTunnelCore
import XCTest

@testable import MullvadSettings

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
