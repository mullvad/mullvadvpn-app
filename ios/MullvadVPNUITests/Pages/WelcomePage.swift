//
//  WelcomePage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-30.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class WelcomePage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .welcomeView
        waitForPageToBeShown()
    }

    @discardableResult func tapAddTimeButton() -> Self {
        app.buttons[AccessibilityIdentifier.purchaseButton].tap()
        return self
    }

    @discardableResult func tapRedeemButton() -> Self {
        app.buttons[AccessibilityIdentifier.redeemVoucherButton].tap()
        return self
    }

    func getAccountNumber() -> String {
        let labelValue = app.staticTexts[AccessibilityIdentifier.welcomeAccountNumberLabel].label
        return labelValue.replacingOccurrences(of: " ", with: "")
    }
}
