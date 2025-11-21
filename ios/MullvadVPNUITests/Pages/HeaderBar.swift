//
//  HeaderBar.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-23.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
