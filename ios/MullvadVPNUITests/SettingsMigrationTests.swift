//
//  SettingsMigrationTests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-03-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class SettingsMigrationTests: BaseUITestCase {
    let customDNSServerIPAddress = "123.123.123.123"
    let wireGuardPort = "4001"

    override class func shouldUninstallAppInTeardown() -> Bool {
        return false
    }

    override func setUp() {
        super.setUp()

        agreeToTermsOfServiceIfShown()
        dismissChangeLogIfShown()

        // Relaunch app so that tests start from a deterministic state
        app.terminate()
        app.launch()
    }

    func testChangeCustomDNSSettings() {
        let hasTimeAccountNumber = getAccountWithTime()

        addTeardownBlock {
            self.deleteTemporaryAccountWithTime(accountNumber: hasTimeAccountNumber)
        }

        logoutIfLoggedIn()
        login(accountNumber: hasTimeAccountNumber)

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapDNSSettingsCell()

        DNSSettingsPage(app)
            .tapEditButton()
            .tapAddAServer()
            .tapEnterIPAddressTextField()
            .enterText(customDNSServerIPAddress)
            .dismissKeyboard()
            .tapUseCustomDNSSwitch()
            .tapDoneButton()
    }

    func testChangeSettings() {
        let hasTimeAccountNumber = getAccountWithTime()

        addTeardownBlock {
            self.deleteTemporaryAccountWithTime(accountNumber: hasTimeAccountNumber)
        }

        logoutIfLoggedIn()
        login(accountNumber: hasTimeAccountNumber)

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapDNSSettingsCell()

        DNSSettingsPage(app)
            .tapDNSContentBlockersHeaderExpandButton()
            .tapBlockAdsSwitch()
            .tapBlockTrackerSwitch()
            .tapBlockMalwareSwitch()
            .tapBlockAdultContentSwitch()
            .tapBlockGamblingSwitch()
            .tapBlockSocialMediaSwitch()
            .tapBackButton()

        VPNSettingsPage(app)
            .tapWireGuardPortsExpandButton()
            .tapCustomWireGuardPortTextField()
            .enterText(wireGuardPort)
            .dismissKeyboard()
            .tapWireGuardPortsExpandButton()
            .tapWireGuardObfuscationExpandButton()
            .tapWireGuardObfuscationOnCell()
            .tapWireGuardObfuscationExpandButton()
            .tapUDPOverTCPPortExpandButton()
            .tapUDPOverTCPPort80Cell()
            .tapUDPOverTCPPortExpandButton()
    }

    func testVerifyCustomDNSSettingsStillChanged() {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapDNSSettingsCell()

        DNSSettingsPage(app)
            .verifyUseCustomDNSSwitchOn()
            .verifyCustomDNSIPAddress(customDNSServerIPAddress)
    }

    func testVerifySettingsStillChanged() {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapDNSSettingsCell()

        DNSSettingsPage(app)
            .tapDNSContentBlockersHeaderExpandButton()
            .verifyBlockAdsSwitchOn()
            .verifyBlockTrackerSwitchOn()
            .verifyBlockMalwareSwitchOn()
            .verifyBlockAdultContentSwitchOn()
            .verifyBlockGamblingSwitchOn()
            .verifyBlockSocialMediaSwitchOn()
            .tapBackButton()

        VPNSettingsPage(app)
            .tapWireGuardPortsExpandButton()
            .verifyCustomWireGuardPortSelected(portNumber: wireGuardPort)
            .tapWireGuardPortsExpandButton()
            .tapWireGuardObfuscationExpandButton()
            .tapWireGuardObfuscationOnCell()
            .verifyWireGuardObfuscationOnSelected()
            .tapWireGuardObfuscationExpandButton()
            .tapUDPOverTCPPortExpandButton()
            .verifyUDPOverTCPPort80Selected()
            .tapBackButton()
    }
}
