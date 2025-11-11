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

        let accountPage = AccountPage(app)

        accountPage
            .tapAddTimeButton()
            .tapAdd30DaysTimeSheetButton()

        try runPaymentFlow(paymentPage: accountPage)

        accountPage
            .dismissThankYouAlert()
    }

    func testMakeInAppPurchaseOnWelcomeScreen() throws {
        LoginPage(app)
            .tapCreateAccountButton()
            .confirmAccountCreationIfVisible()

        let welcomePage = WelcomePage(app)

        welcomePage
            .tapAddTimeButton()
            .tapAdd30DaysTimeSheetButton()

        try runPaymentFlow(paymentPage: welcomePage)

        welcomePage
            .dismissThankYouAlert()
    }

    func testMakeInAppPurchaseOnOutOfTimeScreen() throws {
        LoginPage(app)
            .tapCreateAccountButton()
            .confirmAccountCreationIfVisible()

        // Wait for page to be shown.
        WelcomePage(app)

        // Relaunch to get to out-of-time view.
        app.terminate()
        app.launch()

        let outOfTimePage = OutOfTimePage(app)

        outOfTimePage
            .tapAddTimeButton()
            .tapAdd30DaysTimeSheetButton()

        try runPaymentFlow(paymentPage: outOfTimePage)

        HeaderBar(app)
            .verifyDeviceLabelShown()
    }

    func testInAppPurchaseWithRestoreOnFailedReceiptUpload() throws {
        let firewallAPIClient = FirewallClient()
        firewallAPIClient.removeRules()

        let hasTimeAccountNumber = getAccountWithTime()
        let accountExpiry = try mullvadAPIWrapper.getAccountExpiry(hasTimeAccountNumber)

        addTeardownBlock {
            self.deleteTemporaryAccountWithTime(accountNumber: hasTimeAccountNumber)
            firewallAPIClient.removeRules()
        }

        login(accountNumber: getAccountWithTime())

        HeaderBar(app)
            .tapAccountButton()

        let accountPage = AccountPage(app)

        accountPage
            .verifyPaidUntil(accountExpiry)
            .tapRestorePurchasesButton()
            .dismissAlreadyRestoredPurchasesAlert()
            .verifyPaidUntil(accountExpiry)
            .tapAddTimeButton()
            .tapAdd30DaysTimeSheetButton()

        firewallAPIClient.createRule(try FirewallRule.makeBlockAPIAccessFirewallRule())

        try runPaymentFlow(paymentPage: accountPage)

        accountPage
            .dismissFailedPurchaseAlert()
            .verifyPaidUntil(accountExpiry)

        firewallAPIClient.removeRules()

        accountPage
            .tapRestorePurchasesButton()
            .dismissRestoredPurchasesAlert()

        let newAccountExpiry = try mullvadAPIWrapper.getAccountExpiry(hasTimeAccountNumber)

        accountPage
            .waitForPaidUntil(newAccountExpiry)
    }
}

extension PaymentTests {
    func runPaymentFlow(paymentPage: PaymentPage) throws {
        paymentPage
            .submitSubscribeSheet()

        let flow = paymentPage.determinePaymentFlow()

        switch flow {
        case .confirmAccountSheet:
            print("testMakeInAppPurchase: account flow")

            paymentPage
                .typeCredentialsInAccountSheet(
                    username: try inAppPurchaseUsername,
                    password: try inAppPurchasePassword
                )
                .submitConfirmAccountSheet()
                .submitRenewSubscriptionSheet()
                .submitPurchaseFinishedAlert()

        case .renewSubscriptionAlert:
            print("testMakeInAppPurchase: subscription flow")

            paymentPage
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
