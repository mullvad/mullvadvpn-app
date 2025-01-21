//
//  LoggedOutUITestCase.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-22.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Base class for tests which should start from a logged out state
class LoggedOutUITestCase: BaseUITestCase {
    override func setUp() {
        super.setUp()

        agreeToTermsOfServiceIfShown()
        logoutIfLoggedIn()

        // Relaunch app so that tests start from a deterministic state
        app.terminate()
        app.launch()
    }
}
