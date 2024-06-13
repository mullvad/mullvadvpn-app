//
//  MultihopPromptAlert.swift
//  MullvadVPNUITests
//
//  Created by Mojgan on 2024-06-10.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class MultihopPromptAlert: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.multihopPromptAlert]
        waitForPageToBeShown()
    }

    @discardableResult func tapEnableAnyway() -> Self {
        app.buttons[AccessibilityIdentifier.multihopConfirmAlertEnableButton].tap()
        return self
    }

    @discardableResult func tapBack() -> Self {
        app.buttons[AccessibilityIdentifier.multihopConfirmAlertBackButton].tap()
        return self
    }
}
