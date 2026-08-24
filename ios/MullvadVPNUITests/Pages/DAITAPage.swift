// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import XCTest

class DAITAPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.daitaView]
        waitForPageToBeShown()
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.buttons.matching(identifier: "Settings").allElementsBoundByIndex.last?.tap()
        return self
    }

    @discardableResult func tapEnableDialogButtonIfPresent() -> Self {
        let buttonElement = app.buttons[.daitaConfirmAlertEnableButton]
        if buttonElement.exists {
            buttonElement.tap()
        }
        return self
    }

    @discardableResult func verifyTwoPages() -> Self {
        XCTAssertEqual(app.pageIndicators.firstMatch.value as? String, "page 1 of 2")
        return self
    }

    @discardableResult func tapEnableSwitch() -> Self {
        app.switches[.daitaSwitch].switches.firstMatch.tap()
        return self
    }

    @discardableResult func tapEnableSwitchIfOff() -> Self {
        let switchElement = app.switches[.daitaSwitch].switches.firstMatch

        if switchElement.value as? String == "0" {
            tapEnableSwitch()
        }
        return self
    }
}
