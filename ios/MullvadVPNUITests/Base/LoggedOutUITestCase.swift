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

/// Base class for tests which should start from a logged out state
class LoggedOutUITestCase: BaseUITestCase {
    override class var authenticationState: LaunchArguments.AuthenticationState {
        .forceLoggedOut
    }
    override class var settingsResetPolicy: UITestSettingsResetPolicy {
        .all
    }

    func disableBridgesAccessMethod() {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapAPIAccessCell()

        APIAccessPage(app)
            .getAccessMethodCell(accessibilityId: AccessibilityIdentifier.accessMethodBridgesCell)
            .tap()

        EditAccessMethodPage(app)
            .tapEnableMethodSwitch()
            .tapBackButton()

        // Navigate back to main screen
        let backButton = app.navigationBars.firstMatch.buttons.firstMatch
        backButton.tap()

        SettingsPage(app)
            .tapDoneButton()
    }

}
