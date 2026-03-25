//
//  DAITAPage.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-25.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
        let buttonElement = app.buttons[AccessibilityIdentifier.daitaConfirmAlertEnableButton]
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
        app.switches[AccessibilityIdentifier.daitaSwitch].tap()
        return self
    }

    @discardableResult func tapEnableSwitchIfOff() -> Self {
        let switchElement = app.switches[AccessibilityIdentifier.daitaSwitch]

        if switchElement.value as? String == "0" {
            tapEnableSwitch()
        }
        return self
    }
}
