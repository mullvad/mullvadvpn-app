// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import XCTest

/// Base class for tests that should start from a state of being logged on to an account with time left
class LoggedInWithTimeUITestCase: BaseUITestCase {
    override class var authenticationState: LaunchArguments.AuthenticationState {
        .keepLoggedIn
    }

    override class var settingsResetPolicy: UITestSettingsResetPolicy {
        .only([.settings])
    }

    override func setUp() async throws {
        try await super.setUp()
        guard !isLoggedIn() else { return }

        let accountNumber = await getAccountWithTime()
        login(accountNumber: accountNumber)
    }
}
