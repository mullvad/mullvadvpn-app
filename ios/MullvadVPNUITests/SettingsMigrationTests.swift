//
//  SettingsMigrationTests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-03-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

/// Pre-Release iOS Settings Migration Testing Instructions
///
/// Before releasing a new version, ensure the settings migration process works as expected.
/// Follow these steps to validate that user settings persist correctly across app updates:
///
/// 1. Remove the installed app:
///    Uninstall the current app from the test device to ensure a clean environment.
/// 2. Switch to an older released version:
///    Checkout an app version released approximately 6 months ago for testing migration over a meaningful time span.
/// 3. Run `testChangeCustomDNSSettings`:
///    Modify DNS settings in the older app version to simulate real-world user interactions.
/// 4. Checkout the release branch:
///    Switch to the branch containing the new app version to be released.
///    - Run `testVerifyCustomDNSSettingsStillChanged`:
///      Verify that DNS settings changed in step 3 persist after upgrading.
/// 5. Return to the older version:
///    Checkout the same older version used in step 2 to continue testing additional settings.
/// 6. Run `testChangeVPNSettings`:
///    Modify VPN-related settings in the older app version.
/// 7. Switch back to the release branch:
///    Return to the branch checked out in step 4.
///    - Run `testVerifySettingsStillChanged`:
///      Confirm that VPN settings changed in step 6 persist after upgrading.
///
/// These steps ensure the app's settings migration logic is robust and reliable,
/// providing a seamless user experience during upgrades.
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
            .tapMultihopSwitch()
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
            .verifyMultihopSwitchOn()
    }
}
