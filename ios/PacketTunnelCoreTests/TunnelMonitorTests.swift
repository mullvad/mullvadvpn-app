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

        let pinger = PingerMock(networkStatsReporting: networkCounters) { _, _ in
            return .sendReply()
        }

        let tunnelMonitor = createTunnelMonitor(pinger: pinger, timings: TunnelMonitorTimings())

        tunnelMonitor.onEvent = { event in
            switch event {
            case .connectionEstablished:
                connectedExpectation.fulfill()

            case .connectionLost:
                connectionLostExpectation.fulfill()
            }
        }

        tunnelMonitor.start(probeAddress: .loopback)

        waitForExpectations(timeout: 1)
    }

    func testInitialConnectionTimings() {
        // Setup pinger so that it never receives any replies.
        let pinger = PingerMock(networkStatsReporting: networkCounters) { _, _ in .ignore }

        let timings = TunnelMonitorTimings(
            pingTimeout: .milliseconds(300),
            initialEstablishTimeout: .milliseconds(100),
            connectivityCheckInterval: .milliseconds(100)
        )

        let tunnelMonitor = createTunnelMonitor(pinger: pinger, timings: timings)

        var expectedTimings = [
            timings.initialEstablishTimeout.milliseconds,
            timings.initialEstablishTimeout.milliseconds * 2,
            timings.pingTimeout.milliseconds,
            timings.pingTimeout.milliseconds,
        ]

        // Calculate the amount of time necessary to perform the test.
        var timeout = expectedTimings.reduce(0, +)
        // Add leeway into the total amount of expected wait time.
        timeout += timeout / 2

        let expectation = expectation(description: "Should respect all timings.")
        expectation.expectedFulfillmentCount = expectedTimings.count

        // This date will be used to measure the amount of time elapsed between `.connectionLost` events.
        var startDate = Date()

        tunnelMonitor.onEvent = { [weak tunnelMonitor] event in
            guard case .connectionLost = event else { return }

            switch event {
            case .connectionLost:
                XCTAssertFalse(expectedTimings.isEmpty)

                let expectedDuration = expectedTimings.removeFirst()
                let leeway = expectedDuration / 2

                // Compute amount of time elapsed between `.connectionLost` events.
                let timeElapsed = Int(Date().timeIntervalSince(startDate) * 1000)

                XCTAssertEqual(
                    timeElapsed,
                    expectedDuration,
                    accuracy: leeway,
                    "Expected to report connection loss after \(expectedDuration)-\(expectedDuration + leeway) ms, instead reported it after \(timeElapsed) ms."
                )

                expectation.fulfill()

                if !expectedTimings.isEmpty {
                    startDate = Date()

                    // Continue monitoring by calling start() again.
                    tunnelMonitor?.start(probeAddress: .loopback)
                }

            case .connectionEstablished:
                XCTFail("Connection should fail.")
            }
        }

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
            tunnelDeviceInfo: TunnelDeviceInfoStub(networkStatsProviding: networkCounters),
            timings: timings
        )
    }
}
