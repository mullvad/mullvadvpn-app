//
//  SettingsPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
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

    @discardableResult func tapMultihopCell() -> Self {
        app.tables[AccessibilityIdentifier.settingsTableView]
            .cells[AccessibilityIdentifier.multihopCell]
            .tap()

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
}
