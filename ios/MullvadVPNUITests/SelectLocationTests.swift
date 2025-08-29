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

        XCTAssertTrue(app.staticTexts[AccessibilityIdentifier.daitaFilterPill.asString].exists)
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

        XCTAssertTrue(app.staticTexts[AccessibilityIdentifier.obfuscationFilterPill.asString].exists)
    }
}
