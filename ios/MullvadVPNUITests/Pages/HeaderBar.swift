//
//  HeaderBar.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-23.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class HeaderBar: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .headerBarView
        waitForPageToBeShown()
    }

    @discardableResult func tapAccountButton() -> Self {
        app.buttons[AccessibilityIdentifier.accountButton.rawValue].tap()
        return self
    }

    @discardableResult func tapSettingsButton() -> Self {
        app.buttons[AccessibilityIdentifier.settingsButton.rawValue].tap()
        return self
    }
}
