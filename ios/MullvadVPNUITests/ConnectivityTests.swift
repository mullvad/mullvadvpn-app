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

    override func tearDownWithError() throws {
        super.tearDown()
    }

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

    /// Get the app into a blocked state by connecting to a relay then applying a filter which don't find this relay, then verify that app can still communicate by logging out and verifying that the device was successfully removed
    func testAPIReachableWhenBlocked() {
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

        // Select the first country, it's first city and it's first relay
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

        let deviceName = AccountPage(app).getDeviceName()

        AccountPage(app)
            .tapLogOutButton()

        LoginPage(app)

        verifyDeviceHasBeenRemoved(deviceName: deviceName, accountNumber: hasTimeAccountNumber)
    }
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
