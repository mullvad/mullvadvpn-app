//
//  DaitaPromptAlert.swift
//  MullvadVPNUITests
//
//  Created by Mojgan on 2024-08-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Foundation
import XCTest

class DaitaPromptAlert: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.daitaPromptAlert]
        waitForPageToBeShown()
    }

    @discardableResult func tapEnableAnyway() -> Self {
        app.buttons[AccessibilityIdentifier.daitaConfirmAlertEnableButton].tap()
        return self
    }

    @discardableResult func tapBack() -> Self {
        app.buttons[AccessibilityIdentifier.daitaConfirmAlertBackButton].tap()
        return self
    }
}
