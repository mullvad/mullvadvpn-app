//
//  LeakTests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-05-31.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

class LeakTests: LoggedInWithTimeUITestCase {
    override func tearDown() {
        FirewallAPIClient().removeRules()
    }

    /// Send UDP traffic to a host, connect to relay and make sure while connected to relay no traffic  leaked went directly to the host
    func testNoLeak() throws {
        let targetIPAddress = Networking.getAlwaysReachableIPAddress()
        FirewallAPIClient().createRule(try FirewallRule.makeBlockAllTrafficRule(toIPAddress: targetIPAddress))

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()

        startPacketCapture()
        let trafficGenerator = TrafficGenerator(destinationHost: targetIPAddress, port: 80)
        trafficGenerator.startGeneratingUDPTraffic(interval: 1.0)

        // Keep the tunnel connection for a while
        Thread.sleep(forTimeInterval: 5.0)

        TunnelControlPage(app)
            .tapDisconnectButton()

        // Keep the capture open for a while
        Thread.sleep(forTimeInterval: 5.0)
        trafficGenerator.stopGeneratingUDPTraffic()

        let capturedStreams = stopPacketCapture()
        LeakCheck.assertNoLeaks(streams: capturedStreams, rules: [NoTrafficToHostLeakRule(host: targetIPAddress)])
    }

    /// Send UDP traffic to a host, connect to relay and then disconnect to intentionally leak traffic and make sure that the test catches the leak
    func testShouldLeak() throws {
        let targetIPAddress = Networking.getAlwaysReachableIPAddress()
        FirewallAPIClient().createRule(try FirewallRule.makeBlockAllTrafficRule(toIPAddress: targetIPAddress))

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()

        startPacketCapture()
        let trafficGenerator = TrafficGenerator(destinationHost: targetIPAddress, port: 80)
        trafficGenerator.startGeneratingUDPTraffic(interval: 1.0)

        Thread.sleep(forTimeInterval: 2.0)

        TunnelControlPage(app)
            .tapDisconnectButton()

        // Give it some time to generate traffic outside of tunnel
        Thread.sleep(forTimeInterval: 3.0)

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        // Keep the tunnel connection for a while
        Thread.sleep(forTimeInterval: 5.0)

        app.launch()
        TunnelControlPage(app)
            .tapDisconnectButton()

        // Keep the capture open for a while
        Thread.sleep(forTimeInterval: 5.0)
        trafficGenerator.stopGeneratingUDPTraffic()

        let capturedStreams = stopPacketCapture()
        LeakCheck.assertNoLeaks(streams: capturedStreams, rules: [NoTrafficToHostLeakRule(host: targetIPAddress)])
    }
}
