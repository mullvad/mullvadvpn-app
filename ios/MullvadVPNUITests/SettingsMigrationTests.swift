//
//  SettingsMigrationTests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-03-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

/*
 Settings migration is an exception, it uses four different test plans and a separate workflow
   `ios-end-to-end-tests-settings-migration.yml` which executes the test plans in order,
 do not reinstall the app in between runs but upgrades the app after changing settings:
 * `MullvadVPNUITestsChangeDNSSettings` - Change settings for using custom DNS
 * `MullvadVPNUITestsVerifyDNSSettingsChanged` - Verify custom DNS settings still changed
 * `MullvadVPNUITestsChangeSettings` - Change all settings except custom DNS setting
 * `MullvadVPNUITestsVerifySettingsChanged` - Verify all settings except custom DNS setting still changed
 */
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

    func testChangeVPNSettings() {
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
            .tapBlockGamblingSwitch()
            .tapBlockAdultContentSwitch()
            .tapBlockSocialMediaSwitch()
            .tapBackButton()

        VPNSettingsPage(app)
            .tapWireGuardPortsExpandButton()
            .tapCustomWireGuardPortTextField()
            .enterText(wireGuardPort)
            .dismissKeyboard()
            .tapWireGuardObfuscationExpandButton()
            .tapWireGuardObfuscationOnCell()
            .tapUDPOverTCPPortExpandButton()
            .tapUDPOverTCPPort80Cell()
            .tapQuantumResistantTunnelExpandButton()
            .tapQuantumResistantTunnelOnCell()
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
            .tapWireGuardObfuscationExpandButton()
            .verifyWireGuardObfuscationOnSelected()
            .tapUDPOverTCPPortExpandButton()
            .verifyUDPOverTCPPort80Selected()
            .tapQuantumResistantTunnelExpandButton()
            .verifyQuantumResistantTunnelOnSelected()
    }
}
