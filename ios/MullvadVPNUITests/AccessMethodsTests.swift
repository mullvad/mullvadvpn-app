// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import XCTest

class AccessMethodsTests: LoggedOutUITestCase {
    func testDirect() throws {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapAPIAccessCell()

        APIAccessPage(app)
            .getAccessMethodCell(accessibilityId: AccessibilityIdentifier.accessMethodDirectCell)
            .tap()

        EditAccessMethodPage(app)
            .tapTestMethodButton()
            .verifyTestStatus(.reachable)
    }

    func testBridges() throws {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapAPIAccessCell()

        APIAccessPage(app)
            .getAccessMethodCell(accessibilityId: AccessibilityIdentifier.accessMethodBridgesCell)
            .tap()

        EditAccessMethodPage(app)
            .tapTestMethodButton()
            .verifyTestStatus(.reachable)
    }

    func testEncryptedDNS() throws {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapAPIAccessCell()

        APIAccessPage(app)
            .getAccessMethodCell(accessibilityId: AccessibilityIdentifier.accessMethodEncryptedDNSCell)
            .tap()

        EditAccessMethodPage(app)
            .tapTestMethodButton()
            .verifyTestStatus(.reachable)
    }

    func testCanNotDisableLast() throws {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapAPIAccessCell()

        APIAccessPage(app)
            .getAccessMethodCell(
                accessibilityId: AccessibilityIdentifier.accessMethodDirectCell
            )
            .tap()

        EditAccessMethodPage(app)
            .tapEnableMethodSwitch()
            .tapBackButton()

        APIAccessPage(app)
            .getAccessMethodCell(
                accessibilityId: AccessibilityIdentifier.accessMethodBridgesCell
            )
            .tap()

        EditAccessMethodPage(app)
            .tapEnableMethodSwitch()
            .tapBackButton()

        APIAccessPage(app)
            .getAccessMethodCell(
                accessibilityId: AccessibilityIdentifier.accessMethodEncryptedDNSCell
            )
            .tap()

        EditAccessMethodPage(app)
            .verifySwitchDisabled()
    }
}
