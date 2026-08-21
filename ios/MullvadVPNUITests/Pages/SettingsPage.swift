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
        app
            .cells[AccessibilityIdentifier.apiAccessCell]
            .tap()

        return self
    }

    @discardableResult func tapDAITACell() -> Self {
        app.tables[AccessibilityIdentifier.settingsTableView]
            .cells[AccessibilityIdentifier.daitaCell]
            .tap()

        return self
    }

    @discardableResult func verifyDAITAOn() -> Self {
        let textElement = app.tables[AccessibilityIdentifier.settingsTableView]
            .cells[AccessibilityIdentifier.daitaCell]
            .staticTexts["On"]

        XCTAssertTrue(textElement.exists)

        return self
    }

    @discardableResult func verifyDAITAOff() -> Self {
        let textElement = app.tables[AccessibilityIdentifier.settingsTableView]
            .cells[AccessibilityIdentifier.daitaCell]
            .staticTexts["Off"]

        XCTAssertTrue(textElement.exists)

        return self
    }

    @discardableResult func tapMultihopCell() -> Self {
        app.tables[AccessibilityIdentifier.settingsTableView]
            .cells[AccessibilityIdentifier.multihopCell]
            .tap()

        return self
    }

    @discardableResult func verifyMultihop(state: MultihopState) -> Self {
        let textElement = app.tables[AccessibilityIdentifier.settingsTableView]
            .cells[AccessibilityIdentifier.multihopCell]
            .staticTexts[state.description]

        XCTAssertTrue(textElement.exists)

        return self
    }

    @discardableResult func tapVPNSettingsCell() -> Self {
        app.tables[AccessibilityIdentifier.settingsTableView]
            .cells[AccessibilityIdentifier.vpnSettingsCell]
            .tap()

        return self
    }

    @discardableResult func tapReportAProblemCell() -> Self {
        app.tables[AccessibilityIdentifier.settingsTableView]
            .cells[AccessibilityIdentifier.problemReportCell]
            .tap()

        return self
    }

    @discardableResult func tapLanguageCell() -> Self {
        app.tables[AccessibilityIdentifier.settingsTableView]
            .cells[AccessibilityIdentifier.languageCell]
            .tap()

        return self
    }

    @discardableResult func dismissAlert() -> Self {
        app.buttons["Cancel"]
        return self
    }

    @discardableResult func tapIncludeAllNetworksCell() -> Self {
        app.tables[AccessibilityIdentifier.settingsTableView]
            .cells[AccessibilityIdentifier.includeAllNetworksCell]
            .tap()

        return self
    }
}
