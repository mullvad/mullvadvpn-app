//
//  Payment.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-11-18.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

@MainActor
class Payment {
    enum PaymentFlow {
        case confirmAccountSheet
        case renewSubscriptionAlert
    }

    let app: XCUIApplication
    let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")

    init(_ app: XCUIApplication) {
        self.app = app
    }

    @discardableResult func tapAdd30DaysTimeSheetButton() -> Self {
        // Adding accessibility identifier to the button inside the product sheet would to duplicate
        // button entries, which in turn led to tapping here not working reliably. Using the title
        // as a workaround.
        app.buttons.element(matching: NSPredicate(format: "label BEGINSWITH 'Add 30 days'")).tapWhenHittable()

        return self
    }

    @discardableResult func submitSubscribeSheet() -> Self {
        springboard.buttons["Subscribe"].tapWhenHittable()

        return self
    }

    @discardableResult func typeCredentialsInConfirmAccountSheet(
        username: String,
        password: String
    ) -> Self {
        if springboard.textFields.firstMatch.tapWhenHittable(hardAssertion: false).isEnabled {
            springboard.typeText(username)
        }

        springboard.secureTextFields.firstMatch.tapWhenHittable()
        springboard.typeText(password)

        return self
    }

    @discardableResult func submitConfirmAccountSheet() -> Self {
        springboard.buttons["Confirm"].tapWhenHittable()

        return self
    }

    @discardableResult func submitRenewSubscriptionSheet() -> Self {
        springboard.buttons["Buy"].tapWhenHittable()

        return self
    }

    @discardableResult func submitPurchaseFinishedAlert() -> Self {
        springboard.buttons["OK"].tapWhenHittable()

        return self
    }

    @discardableResult func dismissThankYouAlert() -> Self {
        app.staticTexts["30 days have been added to your account."].wait()
        app.buttons["Got it!"].tap()

        return self
    }

    @discardableResult func determinePaymentFlow() -> PaymentFlow {
        let renewSubscriptionAlert = springboard.alerts["You have subscribed to this in the past"]
            .wait(hardAssertion: false)

        return if renewSubscriptionAlert.exists {
            .renewSubscriptionAlert
        } else {
            .confirmAccountSheet
        }
    }
}
