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
import MullvadSettings
import XCTest

class SettingsPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.settingsContainerView]
        waitForPageToBeShown()
    }

    @discardableResult func tapDoneButton() -> Self {
        app.buttons[AccessibilityIdentifier.settingsDoneButton]
            .tap()

        return self
    }

    @discardableResult func tapAPIAccessCell() -> Self {
        app.buttons[AccessibilityIdentifier.apiAccessCell]
            .tap()

        return self
    }

    @discardableResult func tapDAITACell() -> Self {
        app.buttons[AccessibilityIdentifier.daitaCell]
            .tap()

        return self
    }

    @discardableResult func verifyDAITAOn() -> Self {
        XCTAssertTrue(app.buttons[AccessibilityIdentifier.daitaCell].label.hasSuffix(", On"))

        return self
    }

    @discardableResult func verifyDAITAOff() -> Self {
        XCTAssertTrue(app.buttons[AccessibilityIdentifier.daitaCell].label.hasSuffix(", Off"))

        return self
    }

    @discardableResult func tapMultihopCell() -> Self {
        app.buttons[AccessibilityIdentifier.multihopCell]
            .tap()

        return self
    }

    @discardableResult func verifyMultihop(state: MultihopState) -> Self {
        XCTAssertTrue(app.buttons[AccessibilityIdentifier.multihopCell].label.hasSuffix(", \(state.description)"))

        return self
    }

    @discardableResult func tapVPNSettingsCell() -> Self {
        app.buttons[AccessibilityIdentifier.vpnSettingsCell]
            .tap()

        return self
    }

    @discardableResult func tapReportAProblemCell() -> Self {
        app.buttons[AccessibilityIdentifier.problemReportCell]
            .tap()

        return self
    }

    @discardableResult func tapLanguageCell() -> Self {
        app.buttons[AccessibilityIdentifier.languageCell]
            .tap()

        return self
    }

    @discardableResult func dismissAlert() -> Self {
        app.buttons["Cancel"]
        return self
    }

    @discardableResult func tapIncludeAllNetworksCell() -> Self {
        app.buttons[AccessibilityIdentifier.includeAllNetworksCell]
            .tap()

        return self
    }
}
