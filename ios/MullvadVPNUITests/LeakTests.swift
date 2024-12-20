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
        FirewallClient().removeRules()
        super.tearDown()
    }

    /// Send UDP traffic to a host, connect to relay and make sure while connected to relay no traffic  leaked went directly to the host
    func testNoLeak() throws {
        let targetIPAddress = Networking.getAlwaysReachableIPAddress()
        startPacketCapture()
        let trafficGenerator = TrafficGenerator(destinationHost: targetIPAddress, port: 80)
        trafficGenerator.startGeneratingUDPTraffic(interval: 1.0)

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()

        // Keep the tunnel connection for a while
        Thread.sleep(forTimeInterval: 30.0)

        TunnelControlPage(app)
            .tapDisconnectButton()

        trafficGenerator.stopGeneratingUDPTraffic()

        var capturedStreams = stopPacketCapture()
        // For now cut the beginning and and end of the stream to trim out the part where the tunnel connection was not up
        capturedStreams = PacketCaptureClient.trimPackets(streams: capturedStreams, secondsStart: 8, secondsEnd: 3)
        LeakCheck.assertNoLeaks(streams: capturedStreams, rules: [NoTrafficToHostLeakRule(host: targetIPAddress)])
    }

    /// Send UDP traffic to a host, connect to relay and then disconnect to intentionally leak traffic and make sure that the test catches the leak
    func testShouldLeak() throws {
        let targetIPAddress = Networking.getAlwaysReachableIPAddress()
        startPacketCapture()
        let trafficGenerator = TrafficGenerator(destinationHost: targetIPAddress, port: 80)
        trafficGenerator.startGeneratingUDPTraffic(interval: 1.0)

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()

        Thread.sleep(forTimeInterval: 2.0)

        TunnelControlPage(app)
            .tapDisconnectButton()

        // Give it some time to generate traffic outside of tunnel
        Thread.sleep(forTimeInterval: 5.0)

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        // Keep the tunnel connection for a while
        Thread.sleep(forTimeInterval: 5.0)

        app.launch()
        TunnelControlPage(app)
            .tapDisconnectButton()

        // Keep the capture open for a while
        Thread.sleep(forTimeInterval: 15.0)
        trafficGenerator.stopGeneratingUDPTraffic()

        var capturedStreams = stopPacketCapture()
        // For now cut the beginning and and end of the stream to trim out the part where the tunnel connection was not up
        capturedStreams = PacketCaptureClient.trimPackets(streams: capturedStreams, secondsStart: 8, secondsEnd: 3)
        LeakCheck.assertLeaks(streams: capturedStreams, rules: [NoTrafficToHostLeakRule(host: targetIPAddress)])
    }
}
