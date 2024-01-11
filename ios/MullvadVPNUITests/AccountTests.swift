//
//  MullvadVPNUITests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

final class AccountTests: XCTestCase {
    let noTimeAccountNumber = Bundle(for: AccountTests.self).infoDictionary?["MullvadNoTimeAccountNumber"] as! String
    let hasTimeAccountNumber = Bundle(for: AccountTests.self).infoDictionary?["MullvadHasTimeAccountNumber"] as! String
    let fiveWireGuardKeysAccountNumber = Bundle(for: AccountTests.self)
        .infoDictionary?["MullvadFiveWireGuardKeysAccountNumber"] as! String

    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {}

    func testLogin() throws {
        let app = XCUIApplication()
        app.launch()

        TermsOfServicePage(app)
            .tapAgree()

        Alert(app)
            .tapOkay()

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(self.noTimeAccountNumber)
            .tapAccountNumberSubmitButton()
            .verifySuccessIconShown()
            .verifyDeviceLabelShown()
    }
}
