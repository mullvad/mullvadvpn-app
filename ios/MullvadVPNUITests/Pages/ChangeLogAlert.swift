//
//  ChangeLogAlert.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-02-20.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class ChangeLogAlert: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.changeLogAlert]
        waitForPageToBeShown()
    }

    @discardableResult func tapOkay() -> Self {
        app.buttons[AccessibilityIdentifier.alertOkButton].tap()
        return self
    }
}
