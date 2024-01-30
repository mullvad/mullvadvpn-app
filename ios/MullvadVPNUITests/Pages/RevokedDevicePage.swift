//
//  RevokedDevicePage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-03-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class RevokedDevicePage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .revokedDeviceView
        waitForPageToBeShown()
    }

    @discardableResult func tapGoToLogin() -> Self {
        app.buttons[AccessibilityIdentifier.revokedDeviceLoginButton]
            .tap()

        return self
    }
}
