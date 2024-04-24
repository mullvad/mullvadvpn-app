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
        // Undo enabling block ads in teardown
        addTeardownBlock {
            HeaderBar(self.app)
                .tapSettingsButton()

            SettingsPage(self.app)
                .tapVPNSettingsCell()

            VPNSettingsPage(self.app)
                .tapDNSSettingsCell()

            DNSSettingsPage(self.app)
                .tapDNSContentBlockersHeaderExpandButton()
                .tapBlockAdsSwitchIfOn()
        }

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

        allowAddVPNConfigurationsIfAsked() // Allow adding VPN configurations iOS permission

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()

        try Networking.verifyCannotReachAdServingDomain()

        TunnelControlPage(app)
            .tapDisconnectButton()
    }

    func testAppConnection() throws {
        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()

        try Networking.verifyCanAccessInternet()
        try Networking.verifyConnectedThroughMullvad()
    }

    func testConnectionRetryLogic() throws {
        FirewallAPIClient().removeRules()
        removeFirewallRulesInTearDown = true

        addTeardownBlock {
            TunnelControlPage(self.app)
                .tapSelectLocationButton()

            SelectLocationPage(self.app)
                .tapLocationCell(withName: BaseUITestCase.appDefaultCountry)

            TunnelControlPage(self.app)
                .tapCancelOrDisconnectButton()
        }

        // First get relay IP address
        let relayIPAddress = getDefaultRelayIPAddress()

        // Run actual test
        try FirewallAPIClient().createRule(
            FirewallRule.makeBlockAllTrafficRule(toIPAddress: relayIPAddress)
        )

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCell(withName: BaseUITestCase.testsDefaultRelay)

        // Should be two UDP connection attempts but sometimes only one is shown in the UI
        TunnelControlPage(app)
            .verifyConnectionAttemptsOrder()
            .tapCancelButton()
    }

    func testWireGuardOverTCPManually() throws {
        addTeardownBlock {
            HeaderBar(self.app)
                .tapSettingsButton()

            SettingsPage(self.app)
                .tapVPNSettingsCell()

            VPNSettingsPage(self.app)
                .tapWireGuardObfuscationExpandButton()
                .tapWireGuardObfuscationOffCell()
        }

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
        FirewallAPIClient().removeRules()
        removeFirewallRulesInTearDown = true

        addTeardownBlock {
            TunnelControlPage(self.app)
                .tapSelectLocationButton()

            SelectLocationPage(self.app)
                .tapLocationCell(withName: BaseUITestCase.appDefaultCountry)

            TunnelControlPage(self.app)
                .tapCancelOrDisconnectButton()
        }

        // First get relay IP address
        let relayIPAddress = getDefaultRelayIPAddress()

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
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCell(withName: BaseUITestCase.testsDefaultRelay)

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

            // After editing text field the table is first responder for the first swipe so we need to swipe twice to swipe the modal
            .swipeDownToDismissModal()

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .tapRelayStatusExpandCollapseButton()
            .verifyConnectingToPort("4001")
            .waitForSecureConnectionLabel()
            .tapDisconnectButton()
    }

    /// Get Gothenburg relay IP address and name  by connecting to any Gothenburg relay  and checking relay and IP address the app connects to. Assumes user is logged on and at tunnel control page.
    private func getDefaultRelayIPAddress() -> String {
        TunnelControlPage(app)
            .tapSelectLocationButton()

        if SelectLocationPage(app).locationCellIsExpanded(BaseUITestCase.testsDefaultCountry) {
            // Already expanded - just make sure the correct city cell is selected
            SelectLocationPage(app)
                .tapLocationCell(withName: BaseUITestCase.testsDefaultCity)
        } else {
            SelectLocationPage(app)
                .tapLocationCellExpandButton(withName: BaseUITestCase.testsDefaultCountry)
                .tapLocationCellExpandButton(withName: BaseUITestCase.testsDefaultCity)
                .tapLocationCell(withName: BaseUITestCase.testsDefaultRelay)
        }

        allowAddVPNConfigurationsIfAsked()

        let relayIPAddress = TunnelControlPage(app)
            .waitForSecureConnectionLabel()
            .tapRelayStatusExpandCollapseButton()
            .getInIPAddressFromConnectionStatus()

        TunnelControlPage(app)
            .tapDisconnectButton()

        return relayIPAddress
    }

    func testCustomDNS() throws {
        let dnsServerIPAddress = "8.8.8.8"
        let dnsServerProviderName = "GOOGLE"

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()

        try Networking.verifyCanAccessInternet()

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
