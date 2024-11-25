//
//  DAITAPage.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-25.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
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

    @discardableResult func tapEnableSwitch() -> Self {
        app.switches[AccessibilityIdentifier.daitaSwitch].tap()
        return self
    }

    @discardableResult func tapEnableSwitchIfOn() -> Self {
        let switchElement = app.switches[AccessibilityIdentifier.daitaSwitch]

        if switchElement.value as? String == "1" {
            tapEnableSwitch()
        }
        return self
    }

    @discardableResult func verifyEnableSwitchOn() -> Self {
        let switchElement = app.switches[AccessibilityIdentifier.daitaSwitch]

        guard let switchValue = switchElement.value as? String else {
            XCTFail("Failed to read switch state")
            return self
        }

        XCTAssertEqual(switchValue, "1")
        return self
    }

    @discardableResult func tapDirectOnlySwitch() -> Self {
        app.switches[AccessibilityIdentifier.daitaDirectOnlySwitch].tap()
        return self
    }

    @discardableResult func tapDirectOnlySwitchIfOn() -> Self {
        let switchElement = app.switches[AccessibilityIdentifier.daitaDirectOnlySwitch]

        if switchElement.value as? String == "1" {
            tapEnableSwitch()
        }
        return self
    }

    @discardableResult func verifyDirectOnlySwitchOn() -> Self {
        let switchElement = app.switches[AccessibilityIdentifier.daitaDirectOnlySwitch]

        guard let switchValue = switchElement.value as? String else {
            XCTFail("Failed to read switch state")
            return self
        }

        XCTAssertEqual(switchValue, "1")
        return self
    }
}
