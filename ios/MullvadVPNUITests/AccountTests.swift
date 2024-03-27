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
            .enterText(self.noTimeAccountNumber)
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

    func testLoginToAccountWithTooManyDevices() throws {
        // Setup
        let temporaryAccountNumber = try MullvadAPIWrapper().createAccount()
        try MullvadAPIWrapper().addDevices(5, account: temporaryAccountNumber)

        // Teardown
        addTeardownBlock {
            do {
                try MullvadAPIWrapper().deleteAccount(temporaryAccountNumber)
            } catch {
                XCTFail("Failed to delete account using app API")
            }
        }

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(temporaryAccountNumber)
            .tapAccountNumberSubmitButton()

        DeviceManagementPage(app)
            .tapRemoveDeviceButton(cellIndex: 0)

        DeviceManagementLogOutDeviceConfirmationAlert(app)
            .tapYesLogOutDeviceButton()

        DeviceManagementPage(app)
            .tapContinueWithLoginButton()

        // First taken back to login page and automatically being logged in
        LoginPage(app)
            .verifySuccessIconShown()
            .verifyDeviceLabelShown()

        // And then taken to out of time page because this account don't have any time added to it
        OutOfTimePage(app)
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
