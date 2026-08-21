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
import XCTest

class HeaderBar: Page {
    lazy var accountButton = app.buttons[AccessibilityIdentifier.accountButton]
    lazy var settingsButton = app.buttons[AccessibilityIdentifier.settingsButton]

    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.headerBarView]
        waitForPageToBeShown()
    }

    @discardableResult func tapAccountButton() -> Self {
        accountButton.tap()
        return self
    }

    @discardableResult func tapSettingsButton() -> Self {
        settingsButton.tap()
        return self
    }

    @discardableResult public func verifyDeviceLabelShown() -> Self {
        XCTAssertTrue(
            app.staticTexts[AccessibilityIdentifier.headerDeviceNameLabel]
                .existsAfterWait(),
            "Device name displayed in header"
        )

        return self
    }
}
