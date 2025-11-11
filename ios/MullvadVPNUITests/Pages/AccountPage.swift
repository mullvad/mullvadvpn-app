//
//  AccountPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-23.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

class AccountPage: PaymentPage {
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
        let paidUntilLabelText = app.staticTexts[AccessibilityIdentifier.accountPagePaidUntilLabel].label
        let dateFormatter = DateFormatter()
        dateFormatter.dateStyle = .medium
        dateFormatter.timeStyle = .short

        guard let paidUntilLabelDate = dateFormatter.date(from: paidUntilLabelText) else {
            XCTFail("Failed to convert presented date to Date object")
            return self
        }

        XCTAssertEqual(date.dateWithoutSeconds, paidUntilLabelDate, "Paid until date correct")
        return self
    }

    @discardableResult func waitForPaidUntil(_ date: Date) -> Self {
        app.staticTexts[date.formattedDateString].wait()

        return self
    }

    func waitForLogoutSpinnerToDisappear() {
        let spinnerDisappeared = app.otherElements[.logOutSpinnerAlertView]
            .notExistsAfterWait(timeout: .extremelyLong)
        XCTAssertTrue(spinnerDisappeared, "Log out spinner disappeared")
    }
}

private extension Date {
    var dateWithoutSeconds: Date {
        let calendar = Calendar.current
        var components = calendar.dateComponents([.year, .month, .day, .hour, .minute], from: self)

        // Strip seconds from date, since the app don't display seconds
        components.second = 0

        guard let strippedDate = calendar.date(from: components) else {
            XCTFail("Failed to remove seconds from date")
            return self
        }

        return strippedDate
    }

    var formattedDateString: String {
        let dateFormatter = DateFormatter()
        dateFormatter.dateStyle = .medium
        dateFormatter.timeStyle = .short

        return dateFormatter.string(from: dateWithoutSeconds)
    }
}
