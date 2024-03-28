//
//  RelayTests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-11.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class RelayTests: LoggedInWithTimeUITestCase {
    var removeFirewallRulesInTearDown = false

    override func setUp() {
        super.setUp()

        removeFirewallRulesInTearDown = false
    }

    override func tearDown() {
        super.tearDown()

        if removeFirewallRulesInTearDown {
            FirewallAPIClient().removeRules()
        }
    }

    func testAdBlockingViaDNS() throws {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapDNSSettingsCell()

        DNSSettingsPage(app)
            .tapDNSContentBlockersHeaderExpandButton()
            .tapBlockAdsSwitch()
            .swipeDownToDismissModal()

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurations() // Allow adding VPN configurations iOS permission

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()

        try Networking.verifyCannotReachAdServingDomain()

        TunnelControlPage(app)
            .tapDisconnectButton()
    }

    func testWireGuardOverTCPManually() throws {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapWireGuardObfuscationExpandButton()
            .tapWireGuardObfuscationOnCell()
            .tapBackButton()

        SettingsPage(app)
            .tapDoneButton()

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()

        try Networking.verifyCanAccessInternet()

        TunnelControlPage(app)
            .tapDisconnectButton()
    }

    /// Test automatic switching to TCP is functioning when UDP traffic to relay is blocked. This test first connects to a realy to get the IP address of it, in order to block UDP traffic to this relay.
    func testWireGuardOverTCPAutomatically() throws {
        let wireGuardGot001RelayName = "se-got-wg-001"

        FirewallAPIClient().removeRules()
        removeFirewallRulesInTearDown = true

        // First get relay IP address
        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCellExpandCollapseButton(withName: "Sweden")
            .tapLocationCellExpandCollapseButton(withName: "Gothenburg")
            .tapLocationCell(withName: wireGuardGot001RelayName)

        allowAddVPNConfigurationsIfAsked()

        let relayIPAddress = TunnelControlPage(app)
            .waitForSecureConnectionLabel()
            .tapRelayStatusExpandCollapseButton()
            .getInIPAddressFromConnectionStatus()

        TunnelControlPage(app)
            .tapDisconnectButton()

        // Run actual test
        try FirewallAPIClient().createRule(
            FirewallRule.makeBlockUDPTrafficRule(toIPAddress: relayIPAddress)
        )

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapWireGuardObfuscationExpandButton()
            .tapWireGuardObfuscationAutomaticCell()
            .tapBackButton()

        SettingsPage(app)
            .tapDoneButton()

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        // Should be two UDP connection attempts but sometimes only one is shown in the UI
        TunnelControlPage(app)
            .verifyConnectingOverTCPAfterUDPAttempts()
            .waitForSecureConnectionLabel()
            .tapDisconnectButton()
    }

    func testWireGuardPortSettings() throws {
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapWireGuardPortsExpandButton()
            .tapCustomWireGuardPortTextField()
            .enterText("4001")
            .dismissKeyboard()
            .swipeDownToDismissModal()

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .tapRelayStatusExpandCollapseButton()
            .verifyConnectingToPort("4001")
            .tapDisconnectButton()
    }

    func testCustomDNS() throws {
        let dnsServerIPAddress = "8.8.8.8"
        let dnsServerProviderName = "GOOGLE"

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurations()

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
            .enterText(dnsServerIPAddress)
            .dismissKeyboard()
            .tapUseCustomDNSSwitch()
            .tapDoneButton()

        try Networking.verifyDNSServerProvider(dnsServerProviderName, isMullvad: false)
    }
}
