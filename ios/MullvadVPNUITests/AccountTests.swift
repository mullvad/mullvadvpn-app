//
//  MullvadVPNUITests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-09.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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
            .tryConfirmAccountCreation()

        // Verify welcome page is shown and get account number from it
        let accountNumber = WelcomePage(app).getAccountNumber()

        mullvadAPIWrapper.deleteAccount(accountNumber)
    }

    func testCreateAccountWithLastUsedAccount() throws {
        // Setup
        let temporaryAccountNumber = createTemporaryAccountWithoutTime()

        // Teardown
        addTeardownBlock {
            self.mullvadAPIWrapper.deleteAccount(temporaryAccountNumber)
        }

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(temporaryAccountNumber)
            .tapAccountNumberSubmitButton()

        OutOfTimePage(app)

        HeaderBar(app)
            .tapAccountButton()

        AccountPage(app)
            .tapLogOutButton()

        LoginPage(app)
            .tapCreateAccountButton()
            .confirmAccountCreation()

        // Verify welcome page is shown and get account number from it
        let accountNumber = WelcomePage(app).getAccountNumber()

        self.mullvadAPIWrapper.deleteAccount(accountNumber)
    }

    func testDeleteAccount() throws {
        let accountNumber = createTemporaryAccountWithoutTime()

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
            .tapTextField()
            .enterText(String(accountNumber.suffix(4)))
            .tapDeleteAccountButton()

        // Attempt to login with deleted account and verify that it fails
        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(accountNumber)
            .tapAccountNumberSubmitButton()
            .verifyFailIconShown()
    }

    func testCanNotRemoveCurrentDevice() throws {
        // Setup
        let temporaryAccountNumber = createTemporaryAccountWithoutTime()

        // Teardown
        addTeardownBlock {
            self.mullvadAPIWrapper.deleteAccount(temporaryAccountNumber)
        }

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(temporaryAccountNumber)
            .tapAccountNumberSubmitButton()

        OutOfTimePage(app)

        HeaderBar(app)
            .tapAccountButton()

        AccountPage(app)
            .tapDeviceManagementButton()

        DeviceManagementPage(app)
            .verifyCurrentDeviceExists()
            .verifyCurrentDeviceCannotBeRemoved()
    }

    func testRemoveOtherDevice() throws {
        let otherDevicesCount = 2
        // Setup
        let temporaryAccountNumber = createTemporaryAccountWithoutTime()
        mullvadAPIWrapper.addDevices(otherDevicesCount, account: temporaryAccountNumber)

        // Teardown
        addTeardownBlock {
            self.mullvadAPIWrapper.deleteAccount(temporaryAccountNumber)
        }

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(temporaryAccountNumber)
            .tapAccountNumberSubmitButton()

        OutOfTimePage(app)

        HeaderBar(app)
            .tapAccountButton()

        AccountPage(app)
            .tapDeviceManagementButton()

        DeviceManagementPage(app)
            .waitForDeviceList()
            .verifyRemovableDeviceCount(otherDevicesCount)
            .tapRemoveDeviceButton(cellIndex: 1)

        DeviceManagementLogOutDeviceConfirmationAlert(app)
            .tapYesLogOutDeviceButton()

        DeviceManagementPage(app)
            .waitForDeviceList()
            .waitForNoLoading()
            .verifyRemovableDeviceCount(otherDevicesCount - 1)
    }

    /// Verify logging in works. Will retry x number of times since login request sometimes time out.
    func testLogin() throws {
        let hasTimeAccountNumber = getAccountWithTime()

        addTeardownBlock {
            self.deleteTemporaryAccountWithTime(accountNumber: hasTimeAccountNumber)
        }

        var successIconShown = false
        var retryCount = 0
        let maxRetryCount = 3

        let loginPage = LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(hasTimeAccountNumber)

        repeat {
            successIconShown =
                loginPage
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
            .waitForPageToBeShown()  // Verify still on login page
    }

    func testLoginToAccountWithTooManyDevices() throws {
        // Setup
        let temporaryAccountNumber = createTemporaryAccountWithoutTime()
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
            .waitForDeviceList()
            .tapRemoveDeviceButton(cellIndex: 1)

        DeviceManagementLogOutDeviceConfirmationAlert(app)
            .tapYesLogOutDeviceButton()

        DeviceManagementPage(app)
            .waitForDeviceList()
            .waitForNoLoading()
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
        let newAccountNumber = createTemporaryAccountWithoutTime()
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
            self.deleteTemporaryAccountWithTime(accountNumber: hasTimeAccountNumber)
        }

        login(accountNumber: hasTimeAccountNumber)

        let accountExpiry = try mullvadAPIWrapper.getAccountExpiry(hasTimeAccountNumber)

        HeaderBar(app)
            .tapAccountButton()

        AccountPage(app)
            .verifyPaidUntil(accountExpiry)
    }
}
