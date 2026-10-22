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

    /// make the internet unreachable, by adding a broken access method and disabling everything else
    func breakConnectivity() {
        let customAccessMethodName = "Disable-access-dummy"
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapAPIAccessCell()

        APIAccessPage(app)
            .tapAddButton()

        allowLocalNetworkAccessIfAsked()

        AddAccessMethodPage(app)
            .tapNameCell()
            .enterText(customAccessMethodName)
            .tapTypeCell()
            .tapSOCKS5TypeValueCell()
            .tapServerCell()
            .enterText("123.123.123.123")
            .dismissKeyboard()
            .tapPortCell()
            .enterText("123")
            .dismissKeyboard()
            .tapAddButton()
            .waitForAPIUnreachableLabel()

        AddAccessMethodAPIUnreachableAlert(app)
            .tapSaveButton()

        var accessMethods = APIAccessPage(app).getAccessMethodCells()
        accessMethods.removeLast()
        for cell in accessMethods {
            cell.tap()
            EditAccessMethodPage(app)
                .tapEnableMethodSwitch()
                .tapBackButton()
        }

        SettingsPage(app)
            .swipeDownToDismissModal()

    }

}
