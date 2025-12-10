//
//  PaymentTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-11-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

@available(iOS 26.0, *)
class PaymentTests: LoggedOutUITestCase {
    func testMakeInAppPurchaseOnAccountScreen() throws {
        let accountNumberWithTime = getAccountWithTime()
        let accountExpiry = try mullvadAPIWrapper.getAccountExpiry(accountNumberWithTime)

        addTeardownBlock {
            self.deleteTemporaryAccountWithTime(accountNumber: accountNumberWithTime)
        }

        login(accountNumber: accountNumberWithTime)

        HeaderBar(app)
            .tapAccountButton()

        let accountPage = AccountPage(app)

        accountPage
            .verifyPaidUntil(accountExpiry)
            .finishUnfinishedSandboxPurchases()
            .tapAddTimeButton()
            .tapAdd30DaysTimeSheetButton()

        try runPaymentFlow(paymentPage: accountPage)

        accountPage
            .dismissThankYouAlert()

        try verifyAccountUpdated(accountNumber: accountNumberWithTime, accountExpiry: accountExpiry)
    }

    func testMakeInAppPurchaseOnWelcomeScreen() throws {
        let accountNumber = createAndLogInToNewAccount()

        addTeardownBlock {
            self.deleteTemporaryAccountWithTime(accountNumber: accountNumber)
        }

        HeaderBar(app)
            .tapAccountButton()

        AccountPage(app)
            .finishUnfinishedSandboxPurchases()
            .swipeDownToDismissModal()

        let welcomePage = WelcomePage(app)

        welcomePage
            .tapAddTimeButton()
            .tapAdd30DaysTimeSheetButton()

        try runPaymentFlow(paymentPage: welcomePage)

        // Wait for page to be shown.
        SetUpAccountCompletedPage(app)
    }

    func testMakeInAppPurchaseOnOutOfTimeScreen() throws {
        let accountNumber = createAndLogInToNewAccount()

        addTeardownBlock {
            self.deleteTemporaryAccountWithTime(accountNumber: accountNumber)
        }

        // Relaunch to get to out-of-time view.
        app.terminate()
        app.launch()

        HeaderBar(app)
            .tapAccountButton()

        AccountPage(app)
            .finishUnfinishedSandboxPurchases()
            .swipeDownToDismissModal()

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

        let accountNumberWithTime = getAccountWithTime()
        let accountExpiry = try mullvadAPIWrapper.getAccountExpiry(accountNumberWithTime)

        addTeardownBlock {
            self.deleteTemporaryAccountWithTime(accountNumber: accountNumberWithTime)
            firewallAPIClient.removeRules()
        }

        login(accountNumber: accountNumberWithTime)

        HeaderBar(app)
            .tapAccountButton()

        let accountPage = AccountPage(app)

        accountPage
            .verifyPaidUntil(accountExpiry)
            .finishUnfinishedSandboxPurchases()
            .tapRestorePurchasesButton()
            .dismissAlreadyRestoredPurchasesAlert()
            .verifyPaidUntil(accountExpiry)
            .tapAddTimeButton()
            .tapAdd30DaysTimeSheetButton()

        firewallAPIClient.createRule(
            try FirewallRule.makeBlockAllTrafficRule(toIPAddress: try MullvadAPIWrapper.getAPIIPAddress())
        )

        try runPaymentFlow(paymentPage: accountPage)

        accountPage
            .dismissFailedPurchaseAlert()
            .verifyPaidUntil(accountExpiry)

        firewallAPIClient.removeRules()

        accountPage
            .tapRestorePurchasesButton()
            .dismissRestoredPurchasesAlert()

        try verifyAccountUpdated(accountNumber: accountNumberWithTime, accountExpiry: accountExpiry)
    }
}

@available(iOS 26.0, *)
extension PaymentTests {
    private func createAndLogInToNewAccount() -> String {
        LoginPage(app)
            .tapCreateAccountButton()
            .tryConfirmAccountCreation()

        let welcomePage = WelcomePage(app)
        let accountNumber = welcomePage.getAccountNumber()

        return accountNumber
    }

    private func verifyAccountUpdated(accountNumber: String, accountExpiry: Date) throws {
        let newAccountExpiry = try mullvadAPIWrapper.getAccountExpiry(accountNumber)
        XCTAssertTrue(newAccountExpiry > accountExpiry)

        AccountPage(app)
            .waitForPaidUntil(newAccountExpiry)
    }

    private func runPaymentFlow(paymentPage: PaymentPage) throws {
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

@available(iOS 26.0, *)
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
