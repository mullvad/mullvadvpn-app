//
//  SelectLocationTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-04-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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
            .tapDirectOnlySwitchIfOff()
            .tapEnableDirectOnlyDialogButtonIfPresent()
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
        addTeardownBlock {
            HeaderBar(self.app)
                .tapSettingsButton()

            SettingsPage(self.app)
                .tapMultihopCell()

            MultihopPage(self.app)
                .tapEnableSwitchIfOn()
                .tapBackButton()

            SettingsPage(self.app)
                .tapDoneButton()
        }

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .verifyMultihopOff()
            .tapDoneButton()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapMenuButton()
            .verifyMultihopOff()
            .tapToggleMultihop()
            .tapDoneButton()

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .verifyMultihopOn()
            .tapDoneButton()
    }
}
