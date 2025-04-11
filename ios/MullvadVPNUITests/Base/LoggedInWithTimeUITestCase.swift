//
//  LoggedInUITestCase.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-22.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

/// Base class for tests that should start from a state of being logged on to an account with time left
class LoggedInWithTimeUITestCase: BaseUITestCase {
    var hasTimeAccountNumber: String?

    override func setUp() {
        super.setUp()

        agreeToTermsOfServiceIfShown()
        // Make sure that if a previous test ended up in a state where the app got stuck connecting to a relay
        // does not affect the next test running
        logoutIfLoggedIn()

        hasTimeAccountNumber = getAccountWithTime()

        guard let hasTimeAccountNumber = self.hasTimeAccountNumber else {
            XCTFail("hasTimeAccountNumber unexpectedly not set")
            return
        }

        login(accountNumber: hasTimeAccountNumber)

        // Relaunch app so that tests start from a deterministic state
        app.terminate()
        app.launch()
    }

    override func tearDown() {
        super.tearDown()

        guard let hasTimeAccountNumber = self.hasTimeAccountNumber else {
            XCTFail("hasTimeAccountNumber unexpectedly not set")
            return
        }

        self.deleteTemporaryAccountWithTime(accountNumber: hasTimeAccountNumber)
    }
}
