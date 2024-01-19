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

        self.pageAccessibilityIdentifier = .settingsTableView
    }

    @discardableResult func tapVPNSettingsCell() -> Self {
        app.tables[AccessibilityIdentifier.settingsTableView]
            .cells[AccessibilityIdentifier.preferencesCell]
            .tap()

        return self
    }

    @discardableResult func tapDNSSettingsCell() -> Self {
        app.tables
            .cells[AccessibilityIdentifier.dnsSettings]
            .tap()

        return self
    }

    @discardableResult func tapDNSContentBlockingHeaderExpandButton() -> Self {
        let headerView = app.otherElements[AccessibilityIdentifier.dnsContentBlockersHeaderView]
        let expandButton = headerView.buttons[AccessibilityIdentifier.collapseButton]
        expandButton.tap()

        return self
    }

    @discardableResult func tapBlockAdsSwitch() -> Self {
        app.cells[AccessibilityIdentifier.blockAdvertising]
            .switches[AccessibilityIdentifier.customSwitch]
            .tap()

        return self
    }
}
