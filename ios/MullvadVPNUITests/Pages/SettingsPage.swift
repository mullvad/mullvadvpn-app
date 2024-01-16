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
    override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .settingsTableView
    }

    @discardableResult func tapVPNSettingsCell() -> Self {
        app.tables[AccessibilityIdentifier.settingsTableView.rawValue]
            .cells[AccessibilityIdentifier.preferencesCell.rawValue]
            .tap()

        return self
    }

    @discardableResult func tapDNSSettings() -> Self {
        app.tables[AccessibilityIdentifier.settingsTableView.rawValue]
            .cells[AccessibilityIdentifier.dnsSettings.rawValue]
            .tap()

        return self
    }
}
