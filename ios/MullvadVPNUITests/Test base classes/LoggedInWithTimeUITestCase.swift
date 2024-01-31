//
//  LoggedInUITestCase.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-22.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Base class for tests that should start from a state of being logged on to an account with time left
class LoggedInWithTimeUITestCase: BaseUITestCase {
    override func setUp() {
        super.setUp()

        agreeToTermsOfServiceIfShown()
        logoutIfLoggedIn()

        login(accountNumber: hasTimeAccountNumber)

        // Relaunch app so that tests start from a deterministic state
        app.terminate()
        app.launch()
    }
}
