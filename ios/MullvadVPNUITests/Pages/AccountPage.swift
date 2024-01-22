//
//  AccountPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-23.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class AccountPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .accountView
        waitForPageToBeShown()
    }

    @discardableResult func tapRedeemVoucherButton() -> Self {
        app.buttons[AccessibilityIdentifier.redeemVoucherButton.rawValue].tap()
        return self
    }

    @discardableResult func tapAdd30DaysTimeButton() -> Self {
        app.buttons[AccessibilityIdentifier.purchaseButton.rawValue].tap()
        return self
    }

    @discardableResult func tapRestorePurchasesButton() -> Self {
        app.buttons[AccessibilityIdentifier.restorePurchasesButton.rawValue].tap()
        return self
    }

    @discardableResult func tapLogOutButton() -> Self {
        app.buttons[AccessibilityIdentifier.logoutButton.rawValue].tap()
        return self
    }

    @discardableResult func tapDeleteAccountButton() -> Self {
        app.buttons[AccessibilityIdentifier.deleteButton.rawValue].tap()
        return self
    }
}
