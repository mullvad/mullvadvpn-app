//
//  PaymentTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-11-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

class PaymentTests: LoggedInWithTimeUITestCase {
    let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")

    func testMakeInAppPurchase() async throws {

        HeaderBar(app)
            .tapAccountButton()

        AccountPage(app)
            .tapAdd30DaysTimeButton()
            .tapAdd30DaysTimeSheetButton(timeout: BaseUITestCase.defaultTimeout)

        AccountPage(springboard)

        // if
//            .typeUsernameInPurchaseAlert(try inAppPurchaseUsername)
//            .typePasswordInPurchaseAlert(try inAppPurchasePassword)
//            .submitPurchaseAlert(timeout: BaseUITestCase.defaultTimeout)
            // else

            .submitSubscribeSheet(timeout: BaseUITestCase.defaultTimeout)
            .typePasswordInConfirmSheet(password: try inAppPurchasePassword)
            .submitConfirmSheet()

        // if
            .submitRenewSheet(timeout: BaseUITestCase.defaultTimeout)


        print("here")
    }
}

extension PaymentTests {
    var inAppPurchaseUsername: String {
        get throws {
            try XCTUnwrap(
                Bundle(for: Self.self).infoDictionary?["InAppPurchaseUsername"] as? String,
                "Read in-app purchase username from info.plist"
            )
        }
    }

    var inAppPurchasePassword: String {
        get throws {
            try XCTUnwrap(
                Bundle(for: Self.self).infoDictionary?["InAppPurchasePassword"] as? String,
                "Read in-app purchase password from info.plist"
            )
        }
    }
}
