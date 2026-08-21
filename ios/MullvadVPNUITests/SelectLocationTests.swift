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

class SelectLocationTests: LoggedInWithTimeUITestCase {
    func testEnableDAITA() {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapDAITACell()

        DAITAPage(app)
            .tapEnableSwitchIfOff()
            .tapEnableDialogButtonIfPresent()
            .tapBackButton()

        SettingsPage(app)
            .tapDoneButton()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        XCTAssertTrue(app.buttons[AccessibilityIdentifier.daitaFilterPill.asString].exists)
    }

    func testEnableShadowsocksObfuscation() {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapAntiCensorshipCell()

        AntiCensorshipPage(app)
            .selectObfuscationShadowsocks()
            .tapBackButton()

        VPNSettingsPage(app)
            .tapBackButton()

        SettingsPage(app)
            .tapDoneButton()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        XCTAssertTrue(app.buttons[AccessibilityIdentifier.obfuscationFilterPill.asString].exists)
    }

    func testMultihopToggle() {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .verifyMultihop(state: .whenNeeded)
            .tapDoneButton()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapMenuButton()
            .verifyMultihopState(.whenNeeded)
            .setMultihopState(.always)
            .tapDoneButton()

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .verifyMultihop(state: .always)
            .tapDoneButton()
    }

    func testRecentsEnabled() {
        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapMenuButton()
            .disableRecents()

        SelectLocationPage(app)
            .tapMenuButton()
            .verifyRecentIsDisabled()
            .enableRecents()

        let firstRecentLocationItem = app.buttons
            .matching(NSPredicate(format: "identifier BEGINSWITH %@", "recentListItem"))
            .firstMatch

        XCTAssertTrue(firstRecentLocationItem.exists)

    }
}
