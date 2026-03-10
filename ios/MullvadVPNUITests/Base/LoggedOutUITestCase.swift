//
//  LoggedOutUITestCase.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-22.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
