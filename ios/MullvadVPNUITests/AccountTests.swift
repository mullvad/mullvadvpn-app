//
//  MullvadVPNUITests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

class AccountTests: LoggedOutUITestCase {
    lazy var mullvadAPIWrapper: MullvadAPIWrapper = {
        do {
            // swiftlint:disable:next force_try
            return try! MullvadAPIWrapper()
        }
    }()

    override func setUpWithError() throws {
        continueAfterFailure = false

        try super.setUpWithError()
    }

    func testCreateAccount() throws {
        LoginPage(app)
            .tapCreateAccountButton()

        // Verify welcome page is shown and get account number from it
        let accountNumber = WelcomePage(app).getAccountNumber()

        mullvadAPIWrapper.deleteAccount(accountNumber)
    }

    func testDeleteAccount() throws {
        let accountNumber = mullvadAPIWrapper.createAccount()

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

    /// Verify logging in works. Will retry x number of times since login request sometimes time out.
    func testLogin() throws {
        let hasTimeAccountNumber = getAccountWithTime()

        addTeardownBlock {
            self.returnAccountWithTime(accountNumber: hasTimeAccountNumber)
        }

        var successIconShown = false
        var retryCount = 0
        let maxRetryCount = 3

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(hasTimeAccountNumber)

        repeat {
            successIconShown = LoginPage(app)
                .tapAccountNumberSubmitButton()
                .getSuccessIconShown()

            retryCount += 1
        } while successIconShown == false && retryCount < maxRetryCount

        HeaderBar(app)
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
        let temporaryAccountNumber = mullvadAPIWrapper.createAccount()
        mullvadAPIWrapper.addDevices(5, account: temporaryAccountNumber)

        // Teardown
        addTeardownBlock {
            self.mullvadAPIWrapper.deleteAccount(temporaryAccountNumber)
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

        HeaderBar(app)
            .verifyDeviceLabelShown()

        // And then taken to out of time page because this account don't have any time added to it
        OutOfTimePage(app)
    }

    func testLogOut() throws {
        let newAccountNumber = mullvadAPIWrapper.createAccount()
        login(accountNumber: newAccountNumber)
        XCTAssertEqual(try mullvadAPIWrapper.getDevices(newAccountNumber).count, 1, "Account has one device")

        HeaderBar(app)
            .tapAccountButton()

        AccountPage(app)
            .tapLogOutButton()
            .waitForLogoutSpinnerToDisappear()

        LoginPage(app)

        XCTAssertEqual(try mullvadAPIWrapper.getDevices(newAccountNumber).count, 0, "Account has 0 devices")
        mullvadAPIWrapper.deleteAccount(newAccountNumber)
    }

    func testTimeLeft() throws {
        let hasTimeAccountNumber = getAccountWithTime()

        addTeardownBlock {
            self.returnAccountWithTime(accountNumber: hasTimeAccountNumber)
        }

        login(accountNumber: hasTimeAccountNumber)

        let accountExpiry = try mullvadAPIWrapper.getAccountExpiry(hasTimeAccountNumber)

        HeaderBar(app)
            .tapAccountButton()

        AccountPage(app)
            .verifyPaidUntil(accountExpiry)
    }
}
