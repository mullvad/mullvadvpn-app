//
//  AccountPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-23.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

class AccountPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageElement = app.otherElements[.accountView]
        waitForPageToBeShown()
    }

    @discardableResult func tapRedeemVoucherButton() -> Self {
        app.buttons[AccessibilityIdentifier.redeemVoucherButton.asString].tap()
        return self
    }

    @discardableResult func tapRestorePurchasesButton() -> Self {
        app.buttons[AccessibilityIdentifier.restorePurchasesButton.asString].tap()
        return self
    }

    @discardableResult func tapLogOutButton() -> Self {
        app.buttons[AccessibilityIdentifier.logoutButton.asString].tap()
        return self
    }

    @discardableResult func tapDeleteAccountButton() -> Self {
        app.buttons[AccessibilityIdentifier.deleteButton.asString].tap()
        return self
    }

    @discardableResult func tapDeviceManagementButton() -> Self {
        app.buttons[AccessibilityIdentifier.deviceManagementButton.asString].tap()
        return self
    }

    func getDeviceName() throws -> String {
        let deviceNameLabel = app.otherElements[AccessibilityIdentifier.accountPageDeviceNameLabel]
        return try XCTUnwrap(deviceNameLabel.value as? String, "Failed to read device name from label")
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

        XCTAssertEqual(strippedDate, paidUntilLabelDate, "Paid until date correct")
        return self
    }

    func waitForLogoutSpinnerToDisappear() {
        let spinnerDisappeared = app.otherElements[.logOutSpinnerAlertView]
            .waitForNonExistence(timeout: BaseUITestCase.extremelyLongTimeout)
        XCTAssertTrue(spinnerDisappeared, "Log out spinner disappeared")
    }
}

extension AccountPage {
    @discardableResult func tapAdd30DaysTimeButton() -> Self {
        app.buttons[AccessibilityIdentifier.purchaseButton].tap()
        return self
    }

    @discardableResult func tapAdd30DaysTimeSheetButton(timeout: TimeInterval = 0) -> Self {
        // Adding accessibility identifier to the button inside the product sheet would to duplicate
        // button entries, which in turn led to tapping here not working. Using the title as a
        // workaround.
        app.buttons.element(matching: NSPredicate(format: "label BEGINSWITH 'Add 30 days'"))
            .wait(timeout: timeout)
            .tap()
        return self
    }

    @discardableResult func typeUsernameInPurchaseAlert(_ username: String) -> Self {
//        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        app.alerts.textFields.firstMatch.tap()
        app.typeText(username)
        return self
    }

    @discardableResult func typePasswordInPurchaseAlert(_ password: String) -> Self {
//        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        app.alerts.secureTextFields.firstMatch.tap()
        app.typeText(password)
        return self
    }

    @discardableResult func submitPurchaseAlert(timeout: TimeInterval) -> Self {
//        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        app.buttons.element(matching: NSPredicate(format: "label = 'OK'"))
            .wait(timeout: timeout)
            .tap()
        return self
    }

    @discardableResult func submitSubscribeSheet(timeout: TimeInterval = 0) -> Self {
        let app = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        app.buttons.element(matching: NSPredicate(format: "label = 'Subscribe'"))
            .wait(timeout: timeout)
            .tap()
        return self
    }

    @discardableResult func typePasswordInConfirmSheet(password: String, timeout: TimeInterval = 0) -> Self {
//        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        app.secureTextFields.firstMatch.wait(timeout: timeout).tap()
        app.typeText(password)
        return self
    }

    @discardableResult func submitConfirmSheet(timeout: TimeInterval = 0) -> Self {
//        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        app.buttons["Confirm"].wait(timeout: timeout).tap()
        return self
    }

    @discardableResult func submitRenewSheet(timeout: TimeInterval = 0) -> Self {
//        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        app.buttons["Buy"].wait(timeout: timeout).tap()
        return self
    }
}

extension XCUIElement {
    func wait(timeout: TimeInterval) -> Self {
        XCTAssertTrue(waitForExistence(timeout: timeout))
        return self
    }
}
