//
//  TunnelMonitorTests.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 15/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Network
@testable import PacketTunnelCore
import XCTest

final class TunnelMonitorTests: XCTestCase {
    let networkCounters = NetworkCounters()

    func testShouldDetermineConnectionEstablished() throws {
        let connectedExpectation = expectation(description: "Should report connected.")
        let connectionLostExpectation = expectation(description: "Should not report connection loss")
        connectionLostExpectation.isInverted = true

        let pinger = MockPinger(networkStatsReporting: networkCounters) { _, _ in
            return .sendReply()
        }

        let tunnelMonitor = TunnelMonitor(
            eventQueue: .main,
            pinger: pinger,
            tunnelDeviceInfo: MockTunnelDeviceInfo(networkStatsProviding: networkCounters),
            defaultPathObserver: MockDefaultPathObserver()
        )

        tunnelMonitor.onEvent = { event in
            switch event {
            case .connectionEstablished:
                connectedExpectation.fulfill()

            case .connectionLost:
                connectionLostExpectation.fulfill()

            case .networkReachabilityChanged:
                break
            }
        }

        tunnelMonitor.start(probeAddress: .loopback)

        waitForExpectations(timeout: 1)
    }

    func testInitialConnectionTimings() {
        // Setup pinger so that it never receives any replies.
        let pinger = MockPinger(networkStatsReporting: networkCounters) { _, _ in
            return .ignore
        }

        let tunnelMonitor = TunnelMonitor(
            eventQueue: .main,
            pinger: pinger,
            tunnelDeviceInfo: MockTunnelDeviceInfo(networkStatsProviding: networkCounters),
            defaultPathObserver: MockDefaultPathObserver()
        )

        /*
         Tunnel monitor uses shorter timeout intervals during the initial connection sequence and picks next relay more
         aggressively in order to reduce connection time.

         First connection attempt starts at 4 second timeout, then doubles with each subsequent attempt, while being
         capped at 15s max.
         */
        var expectedTimings = [4, 8, 15, 15]
        let totalAttemptCount = expectedTimings.count

        // Calculate the amount of time necessary to perform the test adding some leeway.
        let timeout = expectedTimings.reduce(1, +)

        let expectation = self.expectation(description: "Should respect all timings.")
        expectation.expectedFulfillmentCount = expectedTimings.count

        // This date will be used to measure the amount of time elapsed between `.connectionLost` events.
        var startDate = Date()

        // Reconnection attempt counter.
        var currentAttempt = 0

        tunnelMonitor.onEvent = { [weak tunnelMonitor] event in
            guard case .connectionLost = event else { return }

            switch event {
            case .connectionLost:
                XCTAssertFalse(expectedTimings.isEmpty)

                let expectedDuration = expectedTimings.removeFirst()

                // Compute amount of time elapsed between `.connectionLost` events rounding it down towards zero.
                let timeElapsed = Int(Date().timeIntervalSince(startDate).rounded(.down))

                currentAttempt += 1

                print("[\(currentAttempt)/\(totalAttemptCount)] \(event), time elapsed: \(timeElapsed)s")

                XCTAssertEqual(
                    timeElapsed,
                    expectedDuration,
                    "Expected to report connection loss after \(expectedDuration)s, instead reported it after \(timeElapsed)s."
                )

                expectation.fulfill()

                if expectedTimings.isEmpty {
                    print("Finished.")
                } else {
                    startDate = Date()

                    print("Continue monitoring.")

                    // Continue monitoring by calling start() again.
                    tunnelMonitor?.start(probeAddress: .loopback)
                }

            case .connectionEstablished:
                XCTFail()

            case .networkReachabilityChanged:
                break
            }
        }

        print("Start monitoring.")

        // Start monitoring.
        tunnelMonitor.start(probeAddress: .loopback)

        waitForExpectations(timeout: TimeInterval(timeout))
    }
}
