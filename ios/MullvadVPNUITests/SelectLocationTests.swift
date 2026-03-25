//
//  SelectLocationTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-04-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
            .tapWireGuardObfuscationExpandButton()
            .tapWireGuardObfuscationShadowsocksCell()
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
            .verifyMultihop(state: .never)
            .tapDoneButton()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapMenuButton()
            .verifyMultihopState(.never)
            .setMultihopState(.always)
            .tapDoneButton()

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .verifyMultihop(state: .never)
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
