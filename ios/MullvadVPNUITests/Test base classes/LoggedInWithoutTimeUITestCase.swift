//
//  LoggedInWithoutTimeUITestCase.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-23.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Base class for tests that should start from a state of being logged on to an account without time left
class LoggedInWithoutTimeUITestCase: BaseUITestCase {
    override func setUp() {
        super.setUp()

        agreeToTermsOfServiceIfShown()
        discardChangeLogIfShown()
        logoutIfLoggedIn()

        login(accountNumber: noTimeAccountNumber)

        // Relaunch app so that tests start from a deterministic state
        app.terminate()
        app.launch()
    }
}
