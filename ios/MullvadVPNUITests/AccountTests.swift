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
        try super.setUpWithError()
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
}
