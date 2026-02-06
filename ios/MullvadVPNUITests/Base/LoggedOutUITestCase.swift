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
    override func setUp() async throws {
        try await super.setUp()

        agreeToTermsOfServiceIfShown()
        logoutIfLoggedIn()

        // Relaunch app so that tests start from a deterministic state
        app.terminate()
        app.launch()
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
