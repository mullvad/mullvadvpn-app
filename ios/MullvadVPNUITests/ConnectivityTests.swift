//
//  ConnectivityTests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-18.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import XCTest

class ConnectivityTests: LoggedOutUITestCase {
    let firewallAPIClient = FirewallClient()

    /// Verifies that the app still functions when API has been blocked
    func testAPIConnectionViaBridges() throws {
        firewallAPIClient.removeRules()
        let hasTimeAccountNumber = getAccountWithTime()

        addTeardownBlock {
            self.deleteTemporaryAccountWithTime(accountNumber: hasTimeAccountNumber)
            self.firewallAPIClient.removeRules()
        }

        try Networking.verifyCanAccessAPI()  // Just to make sure there's no old firewall rule still active
        firewallAPIClient.createRule(try FirewallRule.makeBlockAPIAccessFirewallRule())
        try Networking.verifyCannotAccessAPI()

        var successIconShown = false
        var retryCount = 0
        let maxRetryCount = 3

        let loginPage = LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(hasTimeAccountNumber)

        // After creating firewall rule first login attempt might fail. More attempts are allowed since the app is cycling between three methods.
        repeat {
            successIconShown =
                loginPage
                .tapAccountNumberSubmitButton()
                .getSuccessIconShown()

            if successIconShown == false {
                // Give it some time to show up. App might be waiting for a network connection to timeout.
                loginPage.waitForAccountNumberSubmitButton()
            }

            retryCount += 1
        } while successIconShown == false && retryCount < maxRetryCount

        HeaderBar(app)
            .verifyDeviceLabelShown()
    }

    /// Get the app into a blocked state by connecting to a relay then applying a filter which don't find this relay, then verify that app can still communicate by logging out and verifying that the device was successfully removed
    func testAPIReachableWhenBlocked() throws {
        let hasTimeAccountNumber = getAccountWithTime()
        addTeardownBlock {
            // Reset any filters
            self.login(accountNumber: hasTimeAccountNumber)

            TunnelControlPage(self.app)
                .tapSelectLocationButton()

            let filterCloseButtons = self.app.buttons
                .matching(identifier: AccessibilityIdentifier.relayFilterChipCloseButton.asString)
                .allElementsBoundByIndex

            for filterCloseButton in filterCloseButtons where filterCloseButton.isHittable {
                filterCloseButton.tap()
            }

            // Reset selected location to Sweden
            SelectLocationPage(self.app)
                .tapLocationCell(withName: BaseUITestCase.appDefaultCountry)

            self.allowAddVPNConfigurationsIfAsked()

            TunnelControlPage(self.app)
                .tapCancelOrDisconnectButton()

            self.deleteTemporaryAccountWithTime(accountNumber: hasTimeAccountNumber)
        }

        // Setup. Enter blocked state by connecting to relay and applying filter which relay isn't part of.
        login(accountNumber: hasTimeAccountNumber)

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapMenuButton()
            .tapFilterButton()

        SelectLocationFilterPage(app)
            .tapMullvadOwnershipCell()
            .tapApplyButton()

        // Select the first country, its first city and its first relay
        SelectLocationPage(app)
            .tapLocationCellExpandButton(withName: BaseUITestCase.testsDefaultCountryName)
            // Must be a little specific here in order to avoid using relay services country with experimental relays
            .tapLocationCellExpandButton(withName: BaseUITestCase.testsDefaultMullvadOwnedCityName)
            .tapLocationCell(withName: BaseUITestCase.testsDefaultMullvadOwnedRelayName)

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapMenuButton()
            .tapFilterButton()

        SelectLocationFilterPage(app)
            .tapRentedOwnershipCell()
            .tapApplyButton()

        SelectLocationPage(app)
            .tapDoneButton()

        // Get device name, log out and make sure device was removed as a a way of verifying that the API can be reached
        HeaderBar(app)
            .tapAccountButton()

        let deviceName = try AccountPage(app).getDeviceName()

        AccountPage(app)
            .tapLogOutButton()
            .waitForLogoutSpinnerToDisappear()

        LoginPage(app)

        verifyDeviceHasBeenRemoved(deviceName: deviceName, accountNumber: hasTimeAccountNumber)
    }

    /// Test that the app is functioning when API is down. To simulate API being down we create a dummy access method
    func testAppStillFunctioningWhenAPIDown() throws {
        let hasTimeAccountNumber = getAccountWithTime()
        let customAccessMethodName = "Disable-access-dummy"

        addTeardownBlock {
            HeaderBar(self.app)
                .tapSettingsButton()

            SettingsPage(self.app)
                .tapAPIAccessCell()

            self.toggleAllAccessMethodsEnabledSwitchesIfOff()

            APIAccessPage(self.app)
                .editAccessMethod(customAccessMethodName)

            EditAccessMethodPage(self.app)
                .tapDeleteButton()
                .confirmAccessMethodDeletion()

            self.deleteTemporaryAccountWithTime(accountNumber: hasTimeAccountNumber)
        }

        // Setup. Create a dummy access method to simulate API being down(unreachable)
        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(hasTimeAccountNumber)
            .tapAccountNumberSubmitButton()

        TunnelControlPage(app)

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapAPIAccessCell()

        APIAccessPage(app)
            .tapAddButton()

        allowLocalNetworkAccessIfAsked()

        AddAccessMethodPage(app)
            .tapNameCell()
            .enterText(customAccessMethodName)
            .tapTypeCell()
            .tapSOCKS5TypeValueCell()
            .tapServerCell()
            .enterText("123.123.123.123")
            .dismissKeyboard()
            .tapPortCell()
            .enterText("123")
            .dismissKeyboard()
            .tapAddButton()
            .waitForAPIUnreachableLabel()

        AddAccessMethodAPIUnreachableAlert(app)
            .tapSaveButton()

        disableBuiltinAccessMethods()

        SettingsPage(app)
            .swipeDownToDismissModal()

        // Actual test. Make sure it is possible to connect to a relay
        TunnelControlPage(app)
            .tapConnectButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()

        HeaderBar(app)
            .tapAccountButton()

        // Log out will take long because API cannot be reached
        AccountPage(app)
            .tapLogOutButton()
            .waitForLogoutSpinnerToDisappear()

        // Verify API cannot be reached by doing a login attempt which should fail
        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(hasTimeAccountNumber)
            .tapAccountNumberSubmitButton()
            .verifyFailIconShown()
    }

    func testIfLocalNetworkSharingIsBlocking() throws {
        let skipReason = """
                This test is currently skipped since there is no way to allow local network access for UI tests.
            Since its blocked by the system, there is no way of testing the `Local network sharing` switch.
            Non of these solutions worked: https://developer.apple.com/forums/thread/668729
            """
        try XCTSkipIf(true, skipReason)
        let hasTimeAccountNumber = getAccountWithTime()
        addTeardownBlock {
            self.deleteTemporaryAccountWithTime(accountNumber: hasTimeAccountNumber)
        }
        agreeToTermsOfServiceIfShown()

        login(accountNumber: hasTimeAccountNumber)

        TunnelControlPage(app)
            .tapConnectButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()

        try Networking.verifyCannotAccessLocalNetwork()

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapLocalNetworkSharingSwitch()
            .tapBackButton()

        SettingsPage(app)
            .tapDoneButton()

        TunnelControlPage(app)
            .waitForConnectedLabel()

        try Networking.verifyCanAccessLocalNetwork()
    }

    private func verifyDeviceHasBeenRemoved(deviceName: String, accountNumber: String) {
        do {
            let devices = try MullvadAPIWrapper().getDevices(accountNumber)

            for device in devices where device.name == deviceName {
                XCTFail("Device has not been removed which tells us that the logout was not successful")
            }
        } catch {
            XCTFail("Failed to get devices from app API")
        }
    }

    /// Toggle enabled switch for all existing access methods.
    /// Preconditions:
    /// - The app is currently showing API access view.
    /// - There is one custom access method enabled
    /// - The extra access method is not disabled
    private func disableBuiltinAccessMethods() {
        var accessMethods = APIAccessPage(app).getAccessMethodCells()
        accessMethods.removeLast()
        for cell in accessMethods {
            cell.tap()
            EditAccessMethodPage(app)
                .tapEnableMethodSwitch()
                .tapBackButton()
        }
    }

    /// Toggle enabled switch for all existing access methods if the switch is in off state. It is a precondition that the app is currently showing API access view.
    private func toggleAllAccessMethodsEnabledSwitchesIfOff() {
        for cell in APIAccessPage(app).getAccessMethodCells() {
            cell.tap()
            EditAccessMethodPage(app)
                .tapEnableMethodSwitchIfOff()
                .tapBackButton()
        }
    }
}
