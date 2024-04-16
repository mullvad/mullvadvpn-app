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

    @discardableResult func verifyPaidUntil(_ date: Date) -> Self {
        // Strip seconds from date, since the app don't display seconds
        let calendar = Calendar.current
        var components = calendar.dateComponents([.year, .month, .day, .hour, .minute], from: date)
        components.second = 0
        guard let strippedDate = calendar.date(from: components) else {
            XCTFail("Failed to remove seconds from date")
            return self
        }

        let paidUntilLabelText = app.staticTexts[AccessibilityIdentifier.accountPagePaidUntilLabel].label
        let dateFormatter = DateFormatter()
        dateFormatter.dateStyle = .medium
        dateFormatter.timeStyle = .short

        guard let paidUntilLabelDate = dateFormatter.date(from: paidUntilLabelText) else {
            XCTFail("Failed to convert presented date to Date object")
            return self
        }

        XCTAssertEqual(strippedDate, paidUntilLabelDate)
        return self
    }

    @discardableResult func waitForSpinnerNoLongerShown() -> Self {
        app.otherElements[AccessibilityIdentifier.logOutSpinnerAlertView]
            .waitForNonExistence(timeout: BaseUITestCase.veryLongTimeout)
        return self
    }
}
