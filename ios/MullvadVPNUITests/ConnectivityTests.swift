//
//  ConnectivityTests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import XCTest

class ConnectivityTests: LoggedOutUITestCase {
    let firewallAPIClient = FirewallAPIClient()

    /// Verifies that the app still functions when API has been blocked
    func testAPIConnectionViaBridges() throws {
        firewallAPIClient.removeRules()
        let hasTimeAccountNumber = getAccountWithTime()

        addTeardownBlock {
            self.returnAccountWithTime(accountNumber: hasTimeAccountNumber)
            self.firewallAPIClient.removeRules()
        }

        try Networking.verifyCanAccessAPI() // Just to make sure there's no old firewall rule still active
        firewallAPIClient.createRule(try FirewallRule.makeBlockAPIAccessFirewallRule())
        try Networking.verifyCannotAccessAPI()

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(hasTimeAccountNumber)
            .tapAccountNumberSubmitButton()

        // After creating firewall rule first login attempt might fail. One more attempt is allowed since the app is cycling between two methods.
        let successIconShown = LoginPage(app)
            .getSuccessIconShown()

        if successIconShown {
            HeaderBar(app)
                .verifyDeviceLabelShown()
        } else {
            LoginPage(app)
                .verifyFailIconShown()
                .tapAccountNumberSubmitButton()
                .verifySuccessIconShown()

            HeaderBar(app)
                .verifyDeviceLabelShown()
        }
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
                .matching(identifier: AccessibilityIdentifier.relayFilterChipCloseButton.rawValue)
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

            self.returnAccountWithTime(accountNumber: hasTimeAccountNumber)
        }

        // Setup. Enter blocked state by connecting to relay and applying filter which relay isn't part of.
        login(accountNumber: hasTimeAccountNumber)

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapFilterButton()

        SelectLocationFilterPage(app)
            .tapOwnershipCellExpandButton()
            .tapMullvadOwnershipCell()
            .tapApplyButton()

        // Select the first country, its first city and its first relay
        SelectLocationPage(app)
            .tapCountryLocationCellExpandButton(withIndex: 0)
            .tapCityLocationCellExpandButton(withIndex: 0)
            .tapRelayLocationCell(withIndex: 0)

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapFilterButton()

        SelectLocationFilterPage(app)
            .tapOwnershipCellExpandButton()
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

    // swiftlint:disable function_body_length
    /// Test that the app is functioning when API is down. To simulate API being down we create a dummy access method
    func testAppStillFunctioningWhenAPIDown() throws {
        let hasTimeAccountNumber = getAccountWithTime()

        addTeardownBlock {
            HeaderBar(self.app)
                .tapSettingsButton()

            SettingsPage(self.app)
                .tapAPIAccessCell()

            self.toggleAllAccessMethodsEnabledSwitchesIfOff()
            self.returnAccountWithTime(accountNumber: hasTimeAccountNumber)
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

        toggleAllAccessMethodsEnabledSwitches()

        APIAccessPage(app)
            .tapAddButton()

        allowLocalNetworkAccessIfAsked()

        AddAccessMethodPage(app)
            .tapNameCell()
            .enterText("Disable-access-dummy")
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

        SettingsPage(app)
            .swipeDownToDismissModal()

        // Actual test. Make sure it is possible to connect to a relay
        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()

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

    // swiftlint:enable function_body_length

    /// Toggle enabled switch for all existing access methods. It is a precondition that the app is currently showing API access view.
    private func toggleAllAccessMethodsEnabledSwitches() {
        for cell in APIAccessPage(app).getAccessMethodCells() {
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
