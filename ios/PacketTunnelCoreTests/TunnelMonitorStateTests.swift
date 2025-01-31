//
//  TunnelMonitorStateTests.swift
//  PacketTunnelCoreTests
//
//  Created by Marco Nikic on 2025-02-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
@testable import PacketTunnelCore
import Testing

struct TunnelMonitorStateTests {
    var testDateComponents = DateComponents()
    var defaultPingTimeout: Duration = .milliseconds(500)
    var hearbeatReplyTimeout: Duration = .seconds(1)
    var pingDelay: Duration = .seconds(1)

    init() async throws {
        testDateComponents.day = 10
        testDateComponents.month = 2
        testDateComponents.year = 2025
        testDateComponents.hour = 10
        testDateComponents.calendar = .current
    }

    @Test(arguments: [
        TunnelMonitorConnectionState.stopped,
        TunnelMonitorConnectionState.pendingStart,
        TunnelMonitorConnectionState.recovering,
        TunnelMonitorConnectionState
            .waitingConnectivity,
    ])
    func connectionIsOk(initialState: TunnelMonitorConnectionState) async throws {
        let state = createState(initialState)
        #expect(state.evaluateConnection(now: .now, pingTimeout: .zero) == .ok)
    }

    @Test(arguments: [TunnelMonitorConnectionState.connecting, TunnelMonitorConnectionState.connected])
    func timeoutWhenReplyComesAfterPingTimeoutIn(state: TunnelMonitorConnectionState) async throws {
        var state = createState(state)

        let now = try #require(testDateComponents.date)
        let oneSecondAgo = try #require(testDateComponents.date?.addingTimeInterval(-1))

        // Sent ping a second ago
        state.updatePingStats(sendResult: PingerSendResult(sequenceNumber: 1), now: oneSecondAgo)
        // 0 latency reply
        let timestamp = state.setPingReplyReceived(1, now: oneSecondAgo)
        #expect(timestamp == oneSecondAgo)
        #expect(state.evaluateConnection(now: now, pingTimeout: state.timings.pingTimeout) == .pingTimeout)
    }

    @Test(arguments: [TunnelMonitorConnectionState.connecting, TunnelMonitorConnectionState.connected])
    func evaluateStateBeforeInitialPingIsSent(state: TunnelMonitorConnectionState) async throws {
        let state = createState(state)
        let now = try #require(testDateComponents.date)
        #expect(state.evaluateConnection(now: now, pingTimeout: state.timings.pingTimeout) == .sendInitialPing)
    }

    @Test func evaluateConnectingStateAfterLastRequestWithoutReply() async throws {
        var state = createState(.connecting)

        let now = try #require(testDateComponents.date)
        let nextPingDelay = state.timings.pingDelay + .seconds(1)
        let oneSecondAgo = try #require(testDateComponents.date?.addingTimeInterval(-nextPingDelay.timeInterval))

        // Sent ping a second ago
        state.updatePingStats(sendResult: PingerSendResult(sequenceNumber: 1), now: oneSecondAgo)

        #expect(state.evaluateConnection(now: now, pingTimeout: .seconds(500)) == .sendNextPing)
    }

    @Test func retryHeartbeatPingWhenHeartbeatNotSuspended() async throws {
        var state = createState(.connected)

        // Send a ping and acknowledge it
        let oneSecondAgo = try #require(testDateComponents.date?.addingTimeInterval(-1))
        let now = try #require(testDateComponents.date)
        state.updatePingStats(sendResult: PingerSendResult(sequenceNumber: 1), now: oneSecondAgo)
        _ = state.setPingReplyReceived(1, now: now)

        #expect(state.evaluateConnection(now: now, pingTimeout: .hours(1)) == .retryHeartbeatPing)
    }

    @Test mutating func witnessTrafficFlowingAsExpected() async throws {
        // Ignore heartbeat
        hearbeatReplyTimeout = .hours(1)
        var state = createState(.connected)

        // Send a ping and acknowledge it
        let oneSecondAgo = try #require(testDateComponents.date?.addingTimeInterval(-1))
        let now = try #require(testDateComponents.date)
        state.updatePingStats(sendResult: PingerSendResult(sequenceNumber: 1), now: oneSecondAgo)
        _ = state.setPingReplyReceived(1, now: now)

        #expect(state.evaluateConnection(now: now, pingTimeout: .hours(1)) == .ok)
    }

    @Test func sendHeartbeatPingWhenConnected() async throws {
        var state = createState(.connected)
        state.isHeartbeatSuspended = true

        // Send a ping and acknowledge it
        let tenSecondAgo = try #require(testDateComponents.date?.addingTimeInterval(-10))
        let now = try #require(testDateComponents.date)
        state.updatePingStats(sendResult: PingerSendResult(sequenceNumber: 1), now: tenSecondAgo)
        _ = state.setPingReplyReceived(1, now: now)
        let newNetStats = WgStats(bytesReceived: 100, bytesSent: 100)
        state.updateNetStats(newStats: newNetStats, now: now)

        #expect(state.evaluateConnection(now: now, pingTimeout: .hours(1)) == .sendHeartbeatPing)
    }

    @Test func suspendHeartbeatWhenTrafficIsWitnessed() async throws {
        var state = createState(.connected)

        // Send a ping and acknowledge it
        let tenSecondAgo = try #require(testDateComponents.date?.addingTimeInterval(-10))
        let now = try #require(testDateComponents.date)
        state.updatePingStats(sendResult: PingerSendResult(sequenceNumber: 1), now: tenSecondAgo)
        _ = state.setPingReplyReceived(1, now: tenSecondAgo)

        // Simulate traffic flowing between hearbeat inteval and traffic flow timeout
        let newNetStats = WgStats(bytesReceived: 100, bytesSent: 100)
        let sevenSecondAgo = try #require(testDateComponents.date?.addingTimeInterval(-7))
        state.updateNetStats(newStats: newNetStats, now: sevenSecondAgo)

        #expect(state.evaluateConnection(now: now, pingTimeout: .hours(1)) == .suspendHeartbeat)
    }

    @Test mutating func outboundTrafficTimeout() async throws {
        hearbeatReplyTimeout = .hours(1)

        var state = createState(.connected)
        state.isHeartbeatSuspended = true
        let now = try #require(testDateComponents.date)
        let twoMinutesAgo = try #require(testDateComponents.date?.addingTimeInterval(-120))
        state.updatePingStats(sendResult: PingerSendResult(sequenceNumber: 1), now: twoMinutesAgo)

        let newNetStats = WgStats(bytesReceived: 100, bytesSent: 100)
        state.updateNetStats(newStats: newNetStats, now: twoMinutesAgo)

        _ = state.setPingReplyReceived(1, now: now)

        #expect(state.evaluateConnection(now: now, pingTimeout: .hours(1)) == .trafficTimeout)
    }

    @Test mutating func inboundTrafficTimeout() async throws {
        hearbeatReplyTimeout = .hours(1)

        var state = createState(.connected)
        state.isHeartbeatSuspended = true
        let now = try #require(testDateComponents.date)
        let oneMinuteAgo = try #require(testDateComponents.date?.addingTimeInterval(-60))
        let thirtySecondsAgo = try #require(testDateComponents.date?.addingTimeInterval(-30))
        state.updatePingStats(sendResult: PingerSendResult(sequenceNumber: 1), now: oneMinuteAgo)

        var newNetStats = WgStats(bytesReceived: 10, bytesSent: 10)
        state.updateNetStats(newStats: newNetStats, now: oneMinuteAgo)

        // Simulate bytes sent only, not received
        newNetStats = WgStats(bytesReceived: 0, bytesSent: 20)
        state.updateNetStats(newStats: newNetStats, now: thirtySecondsAgo)

        _ = state.setPingReplyReceived(1, now: now)

        #expect(state.evaluateConnection(now: now, pingTimeout: .hours(1)) == .inboundTrafficTimeout)
    }

    @Test(arguments: [
        TunnelMonitorConnectionState.stopped,
        TunnelMonitorConnectionState.connected,
        TunnelMonitorConnectionState.pendingStart,
        TunnelMonitorConnectionState.recovering,
        TunnelMonitorConnectionState
            .waitingConnectivity,
    ])

    func pingTimeoutIsUnalteredIn(state: TunnelMonitorConnectionState) async throws {
        let state = createState(state)
        #expect(state.getPingTimeout() == defaultPingTimeout)
    }

    func createState(_ initialState: TunnelMonitorConnectionState) -> TunnelMonitorState {
        let timings = TunnelMonitorTimings(
            heartbeatReplyTimeout: hearbeatReplyTimeout,
            pingTimeout: defaultPingTimeout,
            pingDelay: pingDelay,
            initialEstablishTimeout: .milliseconds(50),
            connectivityCheckInterval: .milliseconds(10)
        )

        return TunnelMonitorState(
            connectionState: initialState,
            netStats: WgStats(),
            pingStats: PingStats(),
            timeoutReference: Date(),
            lastSeenRx: nil,
            lastSeenTx: nil,
            isHeartbeatSuspended: false,
            retryAttempt: 0,
            timings: timings
        )
    }
}
