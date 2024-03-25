//
//  DNSSettingsPage.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-03-19.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class DNSSettingsPage: Page {
    @discardableResult override init(_ app: XCUIApplication) {
        super.init(app)

        self.pageAccessibilityIdentifier = .dnsSettings
    }

    private func assertSwitchOn(accessibilityIdentifier: AccessibilityIdentifier) {
        let switchElement = app.cells[accessibilityIdentifier]
            .switches[AccessibilityIdentifier.customSwitch]

        guard let switchValue = switchElement.value as? String else {
            XCTFail("Failed to read switch state")
            return
        }

        XCTAssertEqual(switchValue, "1")
    }

    @discardableResult func tapBackButton() -> Self {
        // Workaround for setting accessibility identifier on navigation bar button being non-trivial
        app.buttons.matching(identifier: "VPN settings").allElementsBoundByIndex.last?.tap()
        return self
    }

    @discardableResult func tapDNSContentBlockersHeaderExpandButton() -> Self {
        let headerView = app.otherElements[AccessibilityIdentifier.dnsContentBlockersHeaderView]
        let expandButton = headerView.buttons[AccessibilityIdentifier.collapseButton]
        expandButton.tap()

        return self
    }

    @discardableResult func tapUseCustomDNSSwitch() -> Self {
        app.cells[AccessibilityIdentifier.dnsSettingsUseCustomDNSCell]
            .switches[AccessibilityIdentifier.customSwitch]
            .tap()

        return self
    }

    @discardableResult func tapBlockAdsSwitch() -> Self {
        app.cells[AccessibilityIdentifier.blockAdvertising]
            .switches[AccessibilityIdentifier.customSwitch]
            .tap()

        return self
    }

    @discardableResult func tapBlockTrackerSwitch() -> Self {
        app.cells[AccessibilityIdentifier.blockTracking]
            .switches[AccessibilityIdentifier.customSwitch]
            .tap()

        return self
    }

    @discardableResult func tapBlockMalwareSwitch() -> Self {
        app.cells[AccessibilityIdentifier.blockMalware]
            .switches[AccessibilityIdentifier.customSwitch]
            .tap()

        return self
    }

    @discardableResult func tapBlockAdultContentSwitch() -> Self {
        app.cells[AccessibilityIdentifier.blockAdultContent]
            .switches[AccessibilityIdentifier.customSwitch]
            .tap()

        return self
    }

    @discardableResult func tapBlockGamblingSwitch() -> Self {
        app.cells[AccessibilityIdentifier.blockGambling]
            .switches[AccessibilityIdentifier.customSwitch]
            .tap()

        return self
    }

    @discardableResult func tapBlockSocialMediaSwitch() -> Self {
        app.cells[AccessibilityIdentifier.blockSocialMedia]
            .switches[AccessibilityIdentifier.customSwitch]
            .tap()

        return self
    }

    @discardableResult func tapEditButton() -> Self {
        app.buttons[AccessibilityIdentifier.dnsSettingsEditButton]
            .tap()
        return self
    }

    @discardableResult func tapDoneButton() -> Self {
        return self.tapEditButton()
    }

    @discardableResult func tapAddAServer() -> Self {
        app.cells[AccessibilityIdentifier.dnsSettingsAddServerCell]
            .tap()
        return self
    }

    @discardableResult func tapEnterIPAddressTextField() -> Self {
        app.textFields[AccessibilityIdentifier.dnsSettingsEnterIPAddressTextField]
            .tap()
        return self
    }

    @discardableResult func verifyBlockAdsSwitchOn() -> Self {
        assertSwitchOn(accessibilityIdentifier: .blockAdvertising)
        return self
    }

    @discardableResult func verifyBlockTrackerSwitchOn() -> Self {
        assertSwitchOn(accessibilityIdentifier: .blockTracking)
        return self
    }

    @discardableResult func verifyBlockMalwareSwitchOn() -> Self {
        assertSwitchOn(accessibilityIdentifier: .blockMalware)
        return self
    }

    @discardableResult func verifyBlockAdultContentSwitchOn() -> Self {
        assertSwitchOn(accessibilityIdentifier: .blockAdultContent)
        return self
    }

    @discardableResult func verifyBlockGamblingSwitchOn() -> Self {
        assertSwitchOn(accessibilityIdentifier: .blockGambling)
        return self
    }

    @discardableResult func verifyBlockSocialMediaSwitchOn() -> Self {
        assertSwitchOn(accessibilityIdentifier: .blockSocialMedia)
        return self
    }

    @discardableResult func verifyUseCustomDNSSwitchOn() -> Self {
        assertSwitchOn(accessibilityIdentifier: .dnsSettingsUseCustomDNSCell)
        return self
    }

    /// Verify that the UI shows stored DNS server IP address same as `ipAddress`. Note that this function assumes there is only one custom DNS server IP address stored.
    @discardableResult func verifyCustomDNSIPAddress(_ ipAddress: String) -> Self {
        let textField = app.textFields[AccessibilityIdentifier.dnsSettingsEnterIPAddressTextField]

        guard let settingIPAddress = textField.value as? String else {
            XCTFail("Failed to read configured DNS IP address")
            return self
        }

        XCTAssertEqual(ipAddress, settingIPAddress)
        return self
    }
}
