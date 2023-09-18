//
//  TunnelMonitorTests.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 15/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
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

        let tunnelMonitor = createTunnelMonitor(pinger: pinger, timings: TunnelMonitorTimings())

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
        let pinger = MockPinger(networkStatsReporting: networkCounters) { _, _ in .ignore }

        let timings = TunnelMonitorTimings(
            pingTimeout: .milliseconds(300),
            initialEstablishTimeout: .milliseconds(100),
            connectivityCheckInterval: .milliseconds(100)
        )

        let tunnelMonitor = createTunnelMonitor(pinger: pinger, timings: timings)

        /*
         Tunnel monitor uses shorter timeout intervals during the initial connection sequence and picks next relay more
         aggressively in order to reduce connection time.

         In reality, default first connection attempt starts at 4 second timeout, then doubles with each subsequent attempt,
         while being capped at 15s max. For tests, however, we start at 100 milliseconds and cap out at 300.
         */
        var expectedTimings = [
            timings.initialEstablishTimeout.milliseconds,
            timings.initialEstablishTimeout.milliseconds * 2,
            timings.pingTimeout.milliseconds,
            timings.pingTimeout.milliseconds,
        ]

        // Calculate the amount of time necessary to perform the test adding some leeway.
        let timeout = expectedTimings.reduce(1000, +)

        let expectation = self.expectation(description: "Should respect all timings.")
        expectation.expectedFulfillmentCount = expectedTimings.count

        // This date will be used to measure the amount of time elapsed between `.connectionLost` events.
        var startDate = Date()

        // Reconnection attempt counter.
        var currentAttempt = 0

        tunnelMonitor.onEvent = { [weak self, weak tunnelMonitor] event in
            guard let self, case .connectionLost = event else { return }

            switch event {
            case .connectionLost:
                XCTAssertFalse(expectedTimings.isEmpty)

                let expectedDuration = expectedTimings.removeFirst()

                // Compute amount of time elapsed between `.connectionLost` events, rounding to nearest 100 milliseconds.
                let timeElapsed = self.roundToHundreds(Int(Date().timeIntervalSince(startDate) * 1000))

                currentAttempt += 1

                print("[\(currentAttempt)/\(expectedTimings.count)] \(event), time elapsed: \(timeElapsed)s")

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
                XCTFail("Connection should fail.")

            case .networkReachabilityChanged:
                break
            }
        }

        print("Start monitoring.")

        // Start monitoring.
        tunnelMonitor.start(probeAddress: .loopback)

        waitForExpectations(timeout: TimeInterval(timeout) / 1000)
    }
}

extension TunnelMonitorTests {
    private func createTunnelMonitor(pinger: PingerProtocol, timings: TunnelMonitorTimings) -> TunnelMonitor {
        return TunnelMonitor(
            eventQueue: .main,
            pinger: pinger,
            tunnelDeviceInfo: MockTunnelDeviceInfo(networkStatsProviding: networkCounters),
            defaultPathObserver: MockDefaultPathObserver(),
            timings: timings
        )
    }

    private func roundToHundreds(_ value: Int) -> Int {
        return (value / 100 * 100) + ((value % 100) / 50 * 100)
    }
}
