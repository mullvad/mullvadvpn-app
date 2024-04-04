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
        addTeardownBlock {
            self.firewallAPIClient.removeRules()
        }

        try Networking.verifyCanAccessAPI() // Just to make sure there's no old firewall rule still active
        firewallAPIClient.createRule(try FirewallRule.makeBlockAPIAccessFirewallRule())
        try Networking.verifyCannotAccessAPI()

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(self.hasTimeAccountNumber)
            .tapAccountNumberSubmitButton()

        // After creating firewall rule first login attempt might fail. One more attempt is allowed since the app is cycling between two methods.
        if isLoggedIn() {
            LoginPage(app)
                .verifySuccessIconShown()
                .verifyDeviceLabelShown()
        } else {
            LoginPage(app)
                .verifyFailIconShown()
                .tapAccountNumberSubmitButton()
                .verifySuccessIconShown()
                .verifyDeviceLabelShown()
        }
    }

    // swiftlint:disable function_body_length
    /// Test that the app is functioning when API is down. To simulate API being down we create a dummy access
    func testAppStillFunctioningWhenAPIDown() throws {
        addTeardownBlock {
            HeaderBar(self.app)
                .tapSettingsButton()

            SettingsPage(self.app)
                .tapAPIAccessCell()

            self.toggleAllAccessMethodsEnabledSwitchIfOff()
        }

        // Setup. Create a dummy access method to simulate API being down(unreachable)
        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(self.hasTimeAccountNumber)
            .tapAccountNumberSubmitButton()

        TunnelControlPage(app)

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapAPIAccessCell()

        toggleAllAccessMethodsEnabledSwitch()

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
            .waitForSpinnerNoLongerShown()

        // Verify API cannot be reached by doing a login attempt which should fail
        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(self.hasTimeAccountNumber)
            .tapAccountNumberSubmitButton()
            .verifyFailIconShown()
    }

    // swiftlint:enable function_body_length

    /// Toggle enabled switch for all existing access methods. It is a precondition that the app is currently showing API access view.
    private func toggleAllAccessMethodsEnabledSwitch() {
        for cell in APIAccessPage(app).getAccessMethodCells() {
            cell.tap()
            EditAccessMethodPage(app)
                .tapEnableMethodSwitch()
                .tapBackButton()
        }
    }

    /// Toggle enabled switch for all existing access methods. It is a precondition that the app is currently showing API access view.
    private func toggleAllAccessMethodsEnabledSwitchIfOff() {
        for cell in APIAccessPage(app).getAccessMethodCells() {
            cell.tap()
            EditAccessMethodPage(app)
                .tapEnableMethodSwitchIfOff()
                .tapBackButton()
        }
    }
}
