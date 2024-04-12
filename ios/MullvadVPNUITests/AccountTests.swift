//
//  MullvadVPNUITests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

class AccountTests: LoggedOutUITestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false

        try super.setUpWithError()
    }

    override func tearDownWithError() throws {}

    func testCreateAccount() throws {
        LoginPage(app)
            .tapCreateAccountButton()

        // Verify welcome page is shown and get account number from it
        let accountNumber = WelcomePage(app).getAccountNumber()

        try MullvadAPIWrapper().deleteAccount(accountNumber)
    }

    func testDeleteAccount() throws {
        let accountNumber = try MullvadAPIWrapper().createAccount()

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(accountNumber)
            .tapAccountNumberSubmitButton()

        OutOfTimePage(app)

        HeaderBar(app)
            .tapAccountButton()

        AccountPage(app)
            .tapDeleteAccountButton()

        AccountDeletionPage(app)
            .enterText(String(accountNumber.suffix(4)))
            .tapDeleteAccountButton()

        // Attempt to login with deleted account and verify that it fails
        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(accountNumber)
            .tapAccountNumberSubmitButton()
            .verifyFailIconShown()
    }

    func testLogin() throws {
        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(self.hasTimeAccountNumber)
            .tapAccountNumberSubmitButton()
            .verifySuccessIconShown()
            .verifyDeviceLabelShown()
    }

    func testLoginWithIncorrectAccountNumber() throws {
        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText("0000000000000000")
            .tapAccountNumberSubmitButton()
            .verifyFailIconShown()
            .waitForPageToBeShown() // Verify still on login page
    }

    func testLogOut() throws {
        let newAccountNumber = try MullvadAPIWrapper().createAccount()
        login(accountNumber: newAccountNumber)
        XCTAssertEqual(try MullvadAPIWrapper().getDevices(newAccountNumber).count, 1)

        HeaderBar(app)
            .tapAccountButton()

        AccountPage(app)
            .tapLogOutButton()

        LoginPage(app)

        XCTAssertEqual(try MullvadAPIWrapper().getDevices(newAccountNumber).count, 0)
        try MullvadAPIWrapper().deleteAccount(newAccountNumber)
    }

    func testTimeLeft() throws {
        login(accountNumber: hasTimeAccountNumber)

        let accountExpiry = try MullvadAPIWrapper().getAccountExpiry(hasTimeAccountNumber)

        HeaderBar(app)
            .tapAccountButton()

        AccountPage(app)
            .verifyPaidUntil(accountExpiry)
    }
}
