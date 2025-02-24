//
//  RelayTests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-11.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

private struct RelayInfo {
    var name: String
    var ipAddress: String
}

class RelayTests: LoggedInWithTimeUITestCase {
    var removeFirewallRulesInTearDown = false

    override func setUp() {
        super.setUp()

        removeFirewallRulesInTearDown = false
    }

    override func tearDown() {
        super.tearDown()

        if removeFirewallRulesInTearDown {
            FirewallClient().removeRules()
        }
    }

    /// Restore default country by selecting it in location selector and immediately disconnecting when app starts connecting to relay in it
    private func restoreDefaultCountry() {
        TunnelControlPage(self.app)
            .tapSelectLocationButton()

        SelectLocationPage(self.app)
            .tapLocationCell(withName: BaseUITestCase.appDefaultCountry)

        TunnelControlPage(self.app)
            .tapCancelOrDisconnectButton()
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
            .tapConnectButton()

        allowAddVPNConfigurationsIfAsked() // Allow adding VPN configurations iOS permission

        TunnelControlPage(app)
            .waitForConnectedLabel()

        try Networking.verifyCannotReachAdServingDomain()

        TunnelControlPage(app)
            .tapDisconnectButton()
    }

    func testAppConnection() throws {
        TunnelControlPage(app)
            .tapConnectButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()

        try Networking.verifyCanAccessInternet()
        try Networking.verifyConnectedThroughMullvad()
    }

    func testConnectionRetryLogic() throws {
        FirewallClient().removeRules()
        removeFirewallRulesInTearDown = true

        addTeardownBlock {
            self.restoreDefaultCountry()
        }

        // First get relay info
        let relayInfo = getDefaultRelayInfo()

        // Run actual test
        try FirewallClient().createRule(
            FirewallRule.makeBlockAllTrafficRule(toIPAddress: relayInfo.ipAddress)
        )

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCellExpandButton(withName: BaseUITestCase.testsDefaultCityName)
            .tapLocationCell(withName: relayInfo.name)

        // Should be two UDP connection attempts but sometimes only one is shown in the UI
        TunnelControlPage(app)
            .tapRelayStatusExpandCollapseButton()
            .verifyConnectionAttemptsOrder()
            .tapCancelButton()
    }

    func testWireGuardOverTCPCustomPort80() throws {
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
            .tapWireGuardObfuscationUdpOverTcpCell()
            .tapUDPOverTCPPortSelectorButton()

        UDPOverTCPObfuscationSettingsPage(app)
            .tapPort80Cell()
            .tapBackButton()

        VPNSettingsPage(app)
            .tapBackButton()

        SettingsPage(app)
            .tapDoneButton()

        // The packet capture has to start before the tunnel is up,
        // otherwise the device cannot reach the in-house router anymore
        startPacketCapture()

        TunnelControlPage(app)
            .tapConnectButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()

        let connectedToIPAddress = TunnelControlPage(app)
            .tapRelayStatusExpandCollapseButton()
            .getInIPv4AddressLabel()

        try Networking.verifyCanAccessInternet()

        try generateTraffic(to: connectedToIPAddress, on: 80, assertProtocol: .TCP)
    }

    func testWireGuardOverShadowsocksCustomPort() throws {
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
            .tapWireGuardObfuscationShadowsocksCell()
            .tapShadowsocksPortSelectorButton()

        ShadowsocksObfuscationSettingsPage(app)
            .tapCustomCell()
            .typeTextIntoCustomField("51900")
            .tapBackButton()

        VPNSettingsPage(app)
            .tapBackButton()

        SettingsPage(app)
            .tapDoneButton()

        // The packet capture has to start before the tunnel is up,
        // otherwise the device cannot reach the in-house router anymore
        startPacketCapture()

        TunnelControlPage(app)
            .tapConnectButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()

        let connectedToIPAddress = TunnelControlPage(app)
            .tapRelayStatusExpandCollapseButton()
            .getInIPv4AddressLabel()

        try Networking.verifyCanAccessInternet()

        try generateTraffic(to: connectedToIPAddress, on: 51900, assertProtocol: .UDP)
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
            .tapWireGuardObfuscationUdpOverTcpCell()
            .tapBackButton()

        SettingsPage(app)
            .tapDoneButton()

        TunnelControlPage(app)
            .tapConnectButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()

        try Networking.verifyCanAccessInternet()

        TunnelControlPage(app)
            .tapDisconnectButton()
    }

    func testWireGuardOverShadowsocksManually() throws {
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
            .tapWireGuardObfuscationShadowsocksCell()
            .tapBackButton()

        SettingsPage(app)
            .tapDoneButton()

        TunnelControlPage(app)
            .tapConnectButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()

        try Networking.verifyCanAccessInternet()

        TunnelControlPage(app)
            .tapDisconnectButton()
    }

    /// Test automatic switching to TCP is functioning when UDP traffic to relay is blocked. This test first connects to a realy to get the IP address of it, in order to block UDP traffic to this relay.
    func testWireGuardOverTCPAutomatically() throws {
        FirewallClient().removeRules()
        removeFirewallRulesInTearDown = true

        addTeardownBlock {
            self.restoreDefaultCountry()
        }

        // First get relay info
        let relayInfo = getDefaultRelayInfo()

        // Run actual test
        try FirewallClient().createRule(
            FirewallRule.makeBlockUDPTrafficRule(toIPAddress: relayInfo.ipAddress)
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
            .tapLocationCellExpandButton(withName: BaseUITestCase.testsDefaultCityName)
            .tapLocationCell(withName: relayInfo.name)

        // Should be two UDP connection attempts but sometimes only one is shown in the UI
        TunnelControlPage(app)
            .tapRelayStatusExpandCollapseButton()
            .verifyConnectingOverTCPAfterUDPAttempts()
            .waitForConnectedLabel()
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
            .tapConnectButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .tapRelayStatusExpandCollapseButton()
            .verifyConnectingToPort("4001")
            .waitForConnectedLabel()
            .tapDisconnectButton()
    }

    func testDAITASettings() throws {
        // Undo enabling DAITA in teardown
        addTeardownBlock {
            HeaderBar(self.app)
                .tapSettingsButton()

            SettingsPage(self.app)
                .tapDAITACell()

            DAITAPage(self.app)
                .tapEnableSwitchIfOn()
        }

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .verifyDAITAOff()
            .tapDAITACell()

        DAITAPage(app)
            .verifyTwoPages()
            .verifyDirectOnlySwitchIsDisabled()
            .tapEnableSwitch()
            .verifyDirectOnlySwitchIsEnabled()
            .tapBackButton()

        SettingsPage(app)
            .verifyDAITAOn()
            .tapDoneButton()

        TunnelControlPage(app)
            .tapConnectButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()
            .verifyConnectingUsingDAITA()
            .tapDisconnectButton()
    }

    func testMultihopSettings() throws {
        // Undo enabling Multihop in teardown
        addTeardownBlock {
            HeaderBar(self.app)
                .tapSettingsButton()

            SettingsPage(self.app)
                .tapMultihopCell()

            MultihopPage(self.app)
                .tapEnableSwitchIfOn()
        }

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .verifyMultihopOff()
            .tapMultihopCell()

        MultihopPage(app)
            .verifyOnePage()
            .tapEnableSwitch()
            .tapBackButton()

        SettingsPage(app)
            .verifyMultihopOn()
            .tapDoneButton()

        TunnelControlPage(app)
            .tapConnectButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()
            .verifyConnectingOverMultihop()
            .tapDisconnectButton()
    }

    func testCustomDNS() throws {
        let dnsServerIPAddress = "8.8.8.8"
        let dnsServerProviderName = "GOOGLE"

        TunnelControlPage(app)
            .tapConnectButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()

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

extension RelayTests {
    /// Connect to a relay in the default country and city, get name and IP address of the relay the app successfully connects to. Assumes user is logged on and at tunnel control page.
    private func getDefaultRelayInfo() -> RelayInfo {
        TunnelControlPage(app)
            .tapSelectLocationButton()

        if SelectLocationPage(app).locationCellIsExpanded(BaseUITestCase.testsDefaultCountryName) {
            // Already expanded - just make sure the correct city cell is selected
            SelectLocationPage(app)
                .tapLocationCell(withName: BaseUITestCase.testsDefaultCityName)
        } else {
            SelectLocationPage(app)
                .tapLocationCellExpandButton(withName: BaseUITestCase.testsDefaultCountryName)
                .tapLocationCell(withName: BaseUITestCase.testsDefaultCityName)
        }

        allowAddVPNConfigurationsIfAsked()

        let relayIPAddress = TunnelControlPage(app)
            .waitForConnectedLabel()
            .tapRelayStatusExpandCollapseButton()
            .getInIPAddressFromConnectionStatus()

        let relayName = TunnelControlPage(app).getCurrentRelayName()

        TunnelControlPage(app)
            .tapDisconnectButton()

        return RelayInfo(name: relayName, ipAddress: relayIPAddress)
    }

    private func generateTraffic(
        to connectedToIPAddress: String,
        on port: Int,
        assertProtocol transportProtocol: NetworkTransportProtocol
    ) throws {
        let targetIPAddress = Networking.getAlwaysReachableIPAddress()
        let trafficGenerator = TrafficGenerator(destinationHost: targetIPAddress, port: 80)
        trafficGenerator.startGeneratingUDPTraffic(interval: 0.1)

        RunLoop.current.run(until: .now + 1)
        trafficGenerator.stopGeneratingUDPTraffic()

        TunnelControlPage(app)
            .tapDisconnectButton()
        let capturedStreams = stopPacketCapture()

        // The capture will contain several streams where `other_addr` contains the IP the device connected to
        // One stream will be for the source port, the other for the destination port
        let streamFromPeeerToRelay = try XCTUnwrap(
            capturedStreams.filter { $0.destinationAddress == connectedToIPAddress && $0.destinationPort == port }.first
        )

        XCTAssertTrue(streamFromPeeerToRelay.transportProtocol == transportProtocol)
    }
} // swiftlint:disable:this file_length
