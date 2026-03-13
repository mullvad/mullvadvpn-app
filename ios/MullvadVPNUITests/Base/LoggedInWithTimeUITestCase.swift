//
//  LoggedInUITestCase.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-22.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

/// Base class for tests that should start from a state of being logged on to an account with time left
class LoggedInWithTimeUITestCase: BaseUITestCase {
    private var hasTimeAccountNumber: String? {
        getAccountWithTime()
    }

    override class var authenticationState: LaunchArguments.AuthenticationState {
        .keepLoggedIn
    }

    override class var settingsResetPolicy: UITestSettingsResetPolicy {
        .only([.settings])
    }

    override func setUp() async throws {
        try await super.setUp()
        guard !isLoggedIn() else { return }
        guard let hasTimeAccountNumber = self.hasTimeAccountNumber else {
            XCTFail("hasTimeAccountNumber unexpectedly not set")
            return
        }
        login(accountNumber: hasTimeAccountNumber)
    }
}
