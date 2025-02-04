//
//  AccountDeletionPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-30.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class AccountDeletionPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.deleteAccountView]
        waitForPageToBeShown()
    }

    @discardableResult func tapTextField() -> Self {
        app.textFields[AccessibilityIdentifier.deleteAccountTextField].tap()
        return self
    }

    @discardableResult func tapDeleteAccountButton() -> Self {
        app.otherElements[.deleteAccountView].buttons[AccessibilityIdentifier.deleteButton].tap()
        return self
    }

    @discardableResult func tapCancelButton() -> Self {
        app.buttons[AccessibilityIdentifier.cancelButton].tap()
        return self
    }
}
