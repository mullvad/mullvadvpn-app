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

    @discardableResult func tapBackButton() -> Self {
        // Workaround due to the way automatically managed back buttons work. Back button needs to be nil for the automatic back button behaviour in iOS, and since its nil we cannot set accessibilityIdentifier for it
        let backButton = app.navigationBars.firstMatch.buttons.firstMatch
        backButton.tap()
        return self
    }
}
