//
//  MullvadVPNUITests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

class AccountTests: BaseTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {}

    func testLogin() throws {
        let app = XCUIApplication()
        app.launch()

        TermsOfServicePage(app)
            .tapAgreeButton()

        Alert(app)
            .tapOkay()

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(self.noTimeAccountNumber)
            .tapAccountNumberSubmitButton()
            .verifySuccessIconShown()
            .verifyDeviceLabelShown()
    }

    func testLoginWithIncorrectAccountNumber() throws {
        let app = XCUIApplication()
        app.launch()

        TermsOfServicePage(app)
            .tapAgreeButton()

        Alert(app)
            .tapOkay()

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText("1234123412341234")
            .tapAccountNumberSubmitButton()
            .verifyFailIconShown()
            .waitForPageToBeShown() // Verify still on login page
    }
}
