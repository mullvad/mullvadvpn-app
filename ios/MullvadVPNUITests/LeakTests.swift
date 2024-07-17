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
    func testNegativeLeaks() throws {
        let testIpAddress = Networking.getAlwaysReachableIPAddress()
        FirewallAPIClient().createRule(try FirewallRule.makeBlockAllTrafficRule(toIPAddress: testIpAddress))
        startPacketCapture()
        let trafficGenerator = TrafficGenerator(destinationHost: testIpAddress, port: 80)
        trafficGenerator.startGeneratingUDPTraffic(interval: 1.0)

        TunnelControlPage(app)
            .tapSecureConnectionButton()

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForSecureConnectionLabel()
        let connectedDate = Date()

        let relayIPAddress = TunnelControlPage(app)
            .getInIPAddressFromConnectionStatus()

        // Keep the tunnel connection for a while
        Thread.sleep(forTimeInterval: 5.0)

        app.launch()
        TunnelControlPage(app)
            .tapDisconnectButton()
        let disconnectedDate = Date()

        // Keep the capture open for a while
        Thread.sleep(forTimeInterval: 3.0)
        trafficGenerator.stopGeneratingUDPTraffic()

        let capturedStreamCollection = stopPacketCapture()

        do {
            let relayConnectionDateInterval = try capturedStreamCollection
                .getConnectedThroughRelayDateInterval(
                    relayIPAddress: relayIPAddress
                )

            // Get traffic from time window of connection with some leeway
            let secondsLeeway = 2.0
            let connectedDateWithLeeway = relayConnectionDateInterval.start.addingTimeInterval(secondsLeeway)
            let disconnectedDateWithLeeway = relayConnectionDateInterval.end.addingTimeInterval(-secondsLeeway)
            let connectedToRelayDateIntervalWithLeeway = DateInterval(
                start: connectedDateWithLeeway,
                end: disconnectedDateWithLeeway
            )
            let connectedThroughRelayStreamCollection = capturedStreamCollection.extractStreamCollectionFrom(
                connectedToRelayDateIntervalWithLeeway,
                cutOffPacketsOverflow: true
            )

            // Treat any traffic to the test IP address during the connected time window as leak
            connectedThroughRelayStreamCollection.dontAllowTrafficFromTestDevice(to: testIpAddress)
            connectedThroughRelayStreamCollection.verifyDontHaveLeaks()
        } catch {
            XCTFail("Unexpectedly didn't find any traffic between test device and relay")
        }
    }
}
