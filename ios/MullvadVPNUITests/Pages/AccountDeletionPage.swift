//
//  AccountDeletionPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-30.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class AccountDeletionPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .deleteAccountView
        waitForPageToBeShown()
    }

    @discardableResult func tapTextField() -> Self {
        app.textFields[AccessibilityIdentifier.deleteAccountTextField].tap()
        return self
    }

    @discardableResult func tapDeleteAccountButton() -> Self {
        guard let pageAccessibilityIdentifier = self.pageAccessibilityIdentifier else {
            XCTFail("Page's accessibility identifier not set")
            return self
        }

        app.otherElements[pageAccessibilityIdentifier].buttons[AccessibilityIdentifier.deleteButton].tap()
        return self
    }

    @discardableResult func tapCancelButton() -> Self {
        app.buttons[AccessibilityIdentifier.cancelButton].tap()
        return self
    }
}
