//
//  TunnelMonitorTests.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 15/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
import MullvadTypes
import Network
@testable import PacketTunnelCore
import XCTest

final class TunnelMonitorTests: XCTestCase {
    let networkCounters = NetworkCounters()

    func testConnectionEstablishedOnPingReply() async throws {
        let connectedExpectation = expectation(description: "Should report connected.")
        let connectionLostExpectation = expectation(description: "Should not report connection loss")
        connectionLostExpectation.isInverted = true

        let pinger = PingerMock(networkStatsReporting: networkCounters) { _, _ in
            return .sendReply()
        }

        let tunnelMonitor = createTunnelMonitor(pinger: pinger, timings: TunnelMonitorTimings())
        await tunnelMonitor.start(probeAddress: .loopback)

        Task {
            for await event in await tunnelMonitor.eventStream {
                switch event {
                case .connectionEstablished:
                    connectedExpectation.fulfill()
                case .connectionLost:
                    connectionLostExpectation.fulfill()
                }
            }
        }

        await fulfillment(of: [connectedExpectation, connectionLostExpectation], timeout: .UnitTest.invertedTimeout)
    }

    /// Verifies that the pinger is stopped when connectivity is not considered reachable,
    /// and started again when connectivity becomes reachable
    func testStartAndStopsMonitoringOnConnectivityUpdates() async {
        let pinger = PingerMock(networkStatsReporting: networkCounters) { _, _ in
            return .sendReply()
        }

        let tunnelMonitor = createTunnelMonitor(pinger: pinger, timings: TunnelMonitorTimings())

        await tunnelMonitor.start(probeAddress: .loopback)

        await tunnelMonitor.handleNetworkPathUpdate(.unsatisfied)
        XCTAssertFalse(pinger.state.isSocketOpen)

        await tunnelMonitor.handleNetworkPathUpdate(.satisfied)
        XCTAssertTrue(pinger.state.isSocketOpen)
    }

    func testSendsConnectionLostEventOnPingTimeout() async {
        let connectionLostExpectation = expectation(description: "Should report connection loss")

        let pinger = PingerMock(networkStatsReporting: networkCounters) { _, _ in
            return .sendReply(reply: .malformed, afterDelay: .milliseconds(10))
        }

        let timings = TunnelMonitorTimings(
            pingTimeout: .milliseconds(300),
            initialEstablishTimeout: .milliseconds(100),
            connectivityCheckInterval: .milliseconds(50)
        )

        let tunnelMonitor = createTunnelMonitor(pinger: pinger, timings: timings)

        Task {
            for await event in await tunnelMonitor.eventStream {
                switch event {
                case .connectionLost:
                    connectionLostExpectation.fulfill()
                default:
                    break
                }
            }
        }

        await tunnelMonitor.start(probeAddress: .loopback)

        pinger.onReply?(.parseError(POSIXError(.ETIMEDOUT)))

        await fulfillment(
            of: [connectionLostExpectation],
            timeout: .UnitTest.invertedTimeout,
            enforceOrder: true
        )
    }

    func testStopStopsPingingAndResetsPingCounter() async throws {
        let pinger = PingerMock(networkStatsReporting: networkCounters) { _, _ in
            return .sendReply(reply: .normal, afterDelay: .zero)
        }

        let timings = TunnelMonitorTimings(
            pingTimeout: .milliseconds(300),
            initialEstablishTimeout: .milliseconds(10),
            connectivityCheckInterval: .milliseconds(50)
        )

        let tunnelMonitor = createTunnelMonitor(pinger: pinger, timings: timings)

        await tunnelMonitor.start(probeAddress: .loopback)
        XCTAssertTrue(pinger.state.isSocketOpen)

        _ = try pinger.send()
        try await Task.sleep(duration: .milliseconds(100))

        var state = await tunnelMonitor.getState()
        XCTAssertTrue(state.netStats.bytesReceived > 0)

        await tunnelMonitor.stop()
        XCTAssertFalse(pinger.state.isSocketOpen)
        state = await tunnelMonitor.getState()
        XCTAssertEqual(0, state.netStats.bytesReceived)
        XCTAssertEqual(0, state.netStats.bytesSent)
    }
}

extension TunnelMonitorTests {
    private func createTunnelMonitor(pinger: PingerProtocol, timings: TunnelMonitorTimings) -> TunnelMonitor {
        return TunnelMonitor(
            pinger: pinger,
            tunnelDeviceInfo: TunnelDeviceInfoStub(networkStatsProviding: networkCounters),
            timings: timings
        )
    }
}
