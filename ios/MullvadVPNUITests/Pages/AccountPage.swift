// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
}

private extension Date {
    var dateWithoutSeconds: Date {
        let calendar = Calendar.current
        var components = calendar.dateComponents([.year, .month, .day, .hour, .minute], from: self)

        // Strip seconds from date, since the app doesn't display seconds.
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
