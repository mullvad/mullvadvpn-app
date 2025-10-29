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

    override func setUp() async throws {
        try await super.setUp()

        removeFirewallRulesInTearDown = false
    }

    override func tearDown() async throws {
        if removeFirewallRulesInTearDown {
            FirewallClient().removeRules()
        }

        try await super.tearDown()
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

        allowAddVPNConfigurationsIfAsked()  // Allow adding VPN configurations iOS permission

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

        // Run actual test
        try FirewallClient().createRule(
            // Block all traffic not going to the router.
            FirewallRule.makeBlockAllTrafficRule(toIPAddress: "8.8.8.8", inverted: true)
        )

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCell(withName: BaseUITestCase.testsDefaultQuicCountryName)

        allowAddVPNConfigurationsIfAsked()

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

        let (connectedToIPAddress, _) = TunnelControlPage(app)
            .tapRelayStatusExpandCollapseButton()
            .getInIPAddressAndPortFromConnectionStatus()

        try Networking.verifyCanAccessInternet()

        try generateTrafficAndDisconnect(from: connectedToIPAddress, searchForPort: 80, assertProtocol: .TCP)
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

        let (connectedToIPAddress, _) = TunnelControlPage(app)
            .tapRelayStatusExpandCollapseButton()
            .getInIPAddressAndPortFromConnectionStatus()

        try Networking.verifyCanAccessInternet()

        try generateTrafficAndDisconnect(from: connectedToIPAddress, searchForPort: 51900, assertProtocol: .UDP)
    }

    func testWireGuardOverTCPManually() throws {
        addTeardownBlock {
            HeaderBar(self.app)
                .tapSettingsButton()

            SettingsPage(self.app)
                .tapVPNSettingsCell()

            VPNSettingsPage(self.app)
                .tapWireGuardObfuscationExpandButton()
                .tapWireGuardObfuscationAutomaticCell()
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

    func testWireGuardOverQuicManually() throws {
        addTeardownBlock {
            HeaderBar(self.app)
                .tapSettingsButton()

            SettingsPage(self.app)
                .tapVPNSettingsCell()

            VPNSettingsPage(self.app)
                .tapWireGuardObfuscationExpandButton()
                .tapWireGuardObfuscationOffCell()
        }

        let deviceIPAddress = try FirewallClient().getDeviceIPAddress()

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapWireGuardObfuscationExpandButton()
            .tapWireGuardObufscationQuicCell()
            .tapBackButton()

        SettingsPage(app)
            .tapDoneButton()

        startPacketCapture()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCellExpandButton(withName: BaseUITestCase.testsDefaultQuicCountryName)
            .tapLocationCellExpandButton(withName: BaseUITestCase.testsDefaultQuicCityName)
            .tapLocationCell(withName: BaseUITestCase.testsDefaultQuicRelayName)

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()

        let (connectedToIPAddress, _) = TunnelControlPage(app)
            .tapRelayStatusExpandCollapseButton()
            .getInIPAddressAndPortFromConnectionStatus()

        let (relayIPAddress, _) = TunnelControlPage(app)
            .getInIPAddressAndPortFromConnectionStatus()

        // Disconnect in order to create firewall rules, otherwise the test router cannot be reached
        TunnelControlPage(app)
            .tapDisconnectButton()

        try FirewallClient().createRule(
            FirewallRule.makeBlockWireGuardTrafficRule(
                fromIPAddress: deviceIPAddress,
                toIPAddress: relayIPAddress
            )
        )

        // The VPN connects despite the wireguard protocol being blocked, QUIC obfuscation is in the works
        TunnelControlPage(app)
            .tapConnectButton()
            .waitForConnectedLabel()

        try Networking.verifyCanAccessInternet()

        try generateTrafficAndDisconnect(from: connectedToIPAddress, searchForPort: 443, assertProtocol: .UDP)
    }

    /// Test automatic switching to TCP is functioning when UDP traffic to relays is blocked.
    func testWireGuardOverTCPAutomatically() throws {
        FirewallClient().removeRules()
        removeFirewallRulesInTearDown = true

        addTeardownBlock {
            self.restoreDefaultCountry()
        }

        // Run actual test
        try FirewallClient().createRule(
            // Block all UDP traffic not going to the router.
            FirewallRule.makeBlockUDPTrafficRule(toIPAddress: "8.8.8.8", inverted: true)
        )

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCell(withName: BaseUITestCase.testsDefaultQuicCountryName)

        allowAddVPNConfigurationsIfAsked()

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
        try disableDaitaInTeardown()

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
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCell(withName: BaseUITestCase.testsNonDAITACountryName)

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()
            .verifyConnectingUsingDAITAThroughMultihop()
            .verifyNotConnectingOverMultihop()
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCell(withName: BaseUITestCase.testsDefaultDAITACountryName)

        TunnelControlPage(app)
            .verifyConnectingUsingDAITA()
            .tapDisconnectButton()
    }

    func testDaitaIncreasesAverageDataConsumption() throws {
        let skipReason = """
                This test is currently skipped due to not being reliable. An issue to fix it has been added here:
                https://linear.app/mullvad/issue/IOS-1348/fix-testdaitaincreasesaveragedataconsumption-flakiness
            """
        try XCTSkipIf(true, skipReason)

        // Verify daita is off
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .verifyDAITAOff()
            .tapDoneButton()

        // Get packet capture #1
        let (firstIPAddress, firstPort, streamWithoutDaita) = try generateTrafficSample()

        // Turn on daita
        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .verifyDAITAOff()
            .tapDAITACell()

        DAITAPage(app)
            .verifyTwoPages()
            .verifyDirectOnlySwitchIsDisabled()
            .tapEnableSwitch()
            .tapBackButton()

        SettingsPage(app)
            .verifyDAITAOn()
            .tapDoneButton()

        try disableDaitaInTeardown()

        // Get packet capture #2
        let (secondIpAddress, secondPort, streamWithDaita) = try generateTrafficSample()

        // Compare packet capture #1 and #2 mean packet size
        let packetStreamWithoutDaita = try XCTUnwrap(
            streamWithoutDaita
                .filter { $0.destinationAddress == firstIPAddress && $0.destinationPort == firstPort }
                .first
        )

        let packetStreamWithDaita = try XCTUnwrap(
            streamWithDaita
                .filter { $0.destinationAddress == secondIpAddress && $0.destinationPort == secondPort }
                .first
        )

        let computeMeanPacketSize: (Stream, Int) -> Int32 = { stream, sampleSize in
            stream.packets[..<sampleSize]
                .map { $0.size }
                .reduce(0, +) / Int32(sampleSize)
        }

        // Sample size might vary a lot, but DAITA is consistently padding enough that 100 samples or so should be good
        // In this case, limit the total sample size to the smallest packet capture
        let maximumSampleSize = min(packetStreamWithoutDaita.packets.count, packetStreamWithDaita.packets.count)
        let meanPacketSizeWithoutDaita = computeMeanPacketSize(packetStreamWithoutDaita, maximumSampleSize)
        let meanPacketSizeWithDaita = computeMeanPacketSize(packetStreamWithDaita, maximumSampleSize)

        XCTAssertTrue(meanPacketSizeWithDaita > meanPacketSizeWithoutDaita)
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

    func testQuantumResistanceSettings() throws {
        addTeardownBlock {
            HeaderBar(self.app)
                .tapSettingsButton()

            SettingsPage(self.app)
                .tapVPNSettingsCell()

            VPNSettingsPage(self.app)
                .tapQuantumResistantTunnelExpandButton()
                .tapQuantumResistantTunnelAutomaticCell()
        }

        TunnelControlPage(app)
            .tapConnectButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .verifyConnectingUsingQuantumResistance()

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapVPNSettingsCell()

        VPNSettingsPage(app)
            .tapQuantumResistantTunnelExpandButton()
            .tapQuantumResistantTunnelOffCell()
            .tapBackButton()

        SettingsPage(app)
            .tapDoneButton()

        TunnelControlPage(app)
            .waitForConnectedLabel()
            .verifyNotConnectingUsingQuantumResistance()
            .tapDisconnectButton()
    }
}

extension RelayTests {
    private func disableDaitaInTeardown() throws {
        // Undo enabling DAITA in teardown
        addTeardownBlock {
            HeaderBar(self.app)
                .tapSettingsButton()

            SettingsPage(self.app)
                .tapDAITACell()

            DAITAPage(self.app)
                .tapEnableSwitchIfOn()
        }
    }

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

        return getRelay()
    }

    /// Connect to a relay in the default country and city, get name and IP address of the relay the app successfully connects to. Assumes user is logged on and at tunnel control page.
    private func getQuicRelayInfo() -> RelayInfo {
        TunnelControlPage(app)
            .tapSelectLocationButton()

        if SelectLocationPage(app).locationCellIsExpanded(BaseUITestCase.testsDefaultQuicCountryName) {
            // Already expanded - just make sure the correct city cell is selected
            SelectLocationPage(app)
                .tapLocationCell(withName: BaseUITestCase.testsDefaultQuicCityName)
        } else {
            SelectLocationPage(app)
                .tapLocationCellExpandButton(withName: BaseUITestCase.testsDefaultQuicCountryName)
                .tapLocationCell(withName: BaseUITestCase.testsDefaultQuicCityName)
        }

        return getRelay()
    }

    private func getRelay() -> RelayInfo {
        allowAddVPNConfigurationsIfAsked()

        let (relayIPAddress, _) = TunnelControlPage(app)
            .waitForConnectedLabel()
            .tapRelayStatusExpandCollapseButton()
            .getInIPAddressAndPortFromConnectionStatus()

        let relayName = TunnelControlPage(app).getCurrentRelayName()

        TunnelControlPage(app)
            .tapDisconnectButton()

        return RelayInfo(name: relayName, ipAddress: relayIPAddress)
    }

    @discardableResult
    private func generateTrafficAndDisconnect(
        from connectedToIPAddress: String,
        searchForPort port: Int,
        duration: TimeInterval = 1,
        assertProtocol transportProtocol: NetworkTransportProtocol
    ) throws -> [Stream] {
        let targetIPAddress = Networking.getAlwaysReachableIPAddress()
        let trafficGenerator = TrafficGenerator(destinationHost: targetIPAddress, port: 80)
        trafficGenerator.startGeneratingUDPTraffic(interval: 0.1)

        RunLoop.current.run(until: .now + duration)
        trafficGenerator.stopGeneratingUDPTraffic()

        TunnelControlPage(app)
            .tapDisconnectButton()
        let capturedStreams = stopPacketCapture()

        // The capture will contain several streams where `other_addr` contains the IP the device connected to
        // One stream will be for the source port, the other for the destination port
        let streamFromPeerToRelay = try XCTUnwrap(
            capturedStreams.filter { $0.destinationAddress == connectedToIPAddress && $0.destinationPort == port }.first
        )

        XCTAssertTrue(streamFromPeerToRelay.transportProtocol == transportProtocol)
        return capturedStreams
    }

    /// Starts a packet capture, connects to a relay, generates synthetic traffic,
    /// disconnects from the relay, and gets a representation of the captured traffic
    private func generateTrafficSample() throws -> (String, Int, [Stream]) {
        startPacketCapture()

        // Connect
        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCell(withName: BaseUITestCase.testsDefaultDAITACountryName)

        allowAddVPNConfigurationsIfAsked()

        // Generate traffic sample
        let (IPAddress, port) = TunnelControlPage(app)
            .waitForConnectedLabel()
            .tapRelayStatusExpandCollapseButton()
            .getInIPAddressAndPortFromConnectionStatus()

        let stream = try generateTrafficAndDisconnect(
            from: IPAddress,
            searchForPort: port,
            duration: 30,
            assertProtocol: .UDP
        )

        return (IPAddress, port, stream)
    }
}
