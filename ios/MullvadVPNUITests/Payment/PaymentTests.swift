//
//  PaymentTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-11-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

class PaymentTests: LoggedOutUITestCase {
    func testMakeInAppPurchaseOnAccountScreen() throws {
        login(accountNumber: getAccountWithTime())

        HeaderBar(app)
            .tapAccountButton()

        AccountPage(app)
            .tapAddTimeButton()

        try runPaymentFlow()

        Payment(app)
            .dismissThankYouAlert()
    }

    func testMakeInAppPurchaseOnWelcomeScreen() throws {
        LoginPage(app)
            .tapCreateAccountButton()

        WelcomePage(app)
            .tapAddTimeButton()

        try runPaymentFlow()

        Payment(app)
            .dismissThankYouAlert()
    }

    func testMakeInAppPurchaseOnOutOfTimeScreen() throws {
        LoginPage(app)
            .tapCreateAccountButton()

        WelcomePage(app).waitForPageToBeShown()

        // Relaunch to get to out-of-time view.
        app.terminate()
        app.launch()

        OutOfTimePage(app)
            .tapAddTimeButton()

        try runPaymentFlow()

        HeaderBar(app)
            .verifyDeviceLabelShown()
    }
}

extension PaymentTests {
    func runPaymentFlow() throws {
        Payment(app)
            .tapAdd30DaysTimeSheetButton()
            .submitSubscribeSheet()

        let flow = Payment(app).determinePaymentFlow()

        switch flow {
        case .confirmAccountSheet:
            print("testMakeInAppPurchase: account flow")

            Payment(app)
                .typeCredentialsInConfirmAccountSheet(
                    username: try inAppPurchaseUsername,
                    password: try inAppPurchasePassword
                )
                .submitConfirmAccountSheet()
                .submitRenewSubscriptionSheet()
                .submitPurchaseFinishedAlert()

        case .renewSubscriptionAlert:
            print("testMakeInAppPurchase: subscription flow")

            Payment(app)
                .submitRenewSubscriptionSheet()
                .submitPurchaseFinishedAlert()
        }
    }
}

extension PaymentTests {
    var inAppPurchaseUsername: String {
        get throws {
            try XCTUnwrap(
                Bundle(for: Self.self).infoDictionary?["IOSInAppPurchaseUsername"] as? String,
                "Read in-app purchase username from info.plist"
            )
        }
    }

    var inAppPurchasePassword: String {
        get throws {
            try XCTUnwrap(
                Bundle(for: Self.self).infoDictionary?["IOSInAppPurchasePassword"] as? String,
                "Read in-app purchase password from info.plist"
            )
        }
    }
}
