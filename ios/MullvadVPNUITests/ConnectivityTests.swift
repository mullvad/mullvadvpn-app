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

    override func setUpWithError() throws {
        super.setUp()
    }

    override func tearDownWithError() throws {
        super.tearDown()
        firewallAPIClient.removeRules()
    }

    /// Verifies that the app still functions when API has been blocked
    func testAPIConnectionViaBridges() throws {
        let app = XCUIApplication()
        app.launch()

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
}
