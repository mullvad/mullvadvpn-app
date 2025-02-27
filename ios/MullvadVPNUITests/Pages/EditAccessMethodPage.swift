//
//  EditAccessMethodPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-04-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class EditAccessMethodPage: Page {
    enum TestStatus {
        case reachable, unreachable, testing
    }

    override init(_ app: XCUIApplication) {
        super.init(app)
        self.pageElement = app.tables[.editAccessMethodView]
        waitForPageToBeShown()
    }

    @discardableResult func tapEnableMethodSwitch() -> Self {
        app.switches[AccessibilityIdentifier.accessMethodEnableSwitch].tap()
        return self
    }

    @discardableResult func tapEnableMethodSwitchIfOff() -> Self {
        let enableMethodSwitch = app.switches[AccessibilityIdentifier.accessMethodEnableSwitch]

        if enableMethodSwitch.value as? String == "0" {
            tapEnableMethodSwitch()
        }

        return self
    }

    @discardableResult func verifyTestStatus(_ status: TestStatus) -> Self {
        switch status {
        case .reachable:
            XCTAssertTrue(app.staticTexts["API reachable"].waitForExistence(timeout: BaseUITestCase.longTimeout))
        case .unreachable:
            XCTAssertTrue(app.staticTexts["API unreachable"].waitForExistence(timeout: BaseUITestCase.longTimeout))
        case .testing:
            XCTAssertTrue(app.staticTexts["Testing..."].waitForExistence(timeout: BaseUITestCase.longTimeout))
        }

        return self
    }

    @discardableResult func tapTestMethodButton() -> Self {
        app.buttons[AccessibilityIdentifier.accessMethodTestButton].tap()
        return self
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround due to the way automatically managed back buttons work. Back button needs to be nil for the automatic back button behaviour in iOS, and since its nil we cannot set accessibilityIdentifier for it
        let backButton = app.navigationBars.firstMatch.buttons.firstMatch
        backButton.tap()
        return self
    }
}
