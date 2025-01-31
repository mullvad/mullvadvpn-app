//
//  TunnelMonitorState.swift
//  PacketTunnelCore
//
//  Created by Marco Nikic on 2024-02-06.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Connection state.
public enum TunnelMonitorConnectionState {
    /// Initialized and doing nothing.
    case stopped

    /// Preparing to start.
    /// Intermediate state before receiving the first path update.
    case pendingStart

    /// Establishing connection.
    case connecting

    /// Connection is established.
    case connected

    /// Delegate is recovering connection.
    /// Delegate has to call `start(probeAddress:)` to complete recovery and resume monitoring.
    case recovering

    /// Waiting for network connectivity.
    case waitingConnectivity
}

enum ConnectionEvaluation {
    case ok
    case sendInitialPing
    case sendNextPing
    case sendHeartbeatPing
    case retryHeartbeatPing
    case suspendHeartbeat
    case inboundTrafficTimeout
    case trafficTimeout
    case pingTimeout
}

/// Tunnel monitor state.
public struct TunnelMonitorState {
    /// Current connection state.
    var connectionState: TunnelMonitorConnectionState

    /// Network counters.
    var netStats = WgStats()

    /// Ping stats.
    var pingStats = PingStats()

    /// Reference date used to determine if timeout has occurred.
    var timeoutReference = Date()

    /// Last seen change in rx counter.
    var lastSeenRx: Date?

    /// Last seen change in tx counter.
    var lastSeenTx: Date?

    /// Whether periodic heartbeat is suspended.
    var isHeartbeatSuspended = false

    /// Retry attempt.
    var retryAttempt: UInt32

    // Timings and timeouts.
    let timings: TunnelMonitorTimings

    public init(
        connectionState: TunnelMonitorConnectionState = .stopped,
        netStats: WgStats = WgStats(),
        pingStats: PingStats = PingStats(),
        timeoutReference: Date = Date(),
        lastSeenRx: Date? = nil,
        lastSeenTx: Date? = nil,
        isHeartbeatSuspended: Bool = false,
        retryAttempt: UInt32 = 0,
        timings: TunnelMonitorTimings
    ) {
        self.connectionState = connectionState
        self.netStats = netStats
        self.pingStats = pingStats
        self.timeoutReference = timeoutReference
        self.lastSeenRx = lastSeenRx
        self.lastSeenTx = lastSeenTx
        self.isHeartbeatSuspended = isHeartbeatSuspended
        self.retryAttempt = retryAttempt
        self.timings = timings
    }

    func evaluateConnection(now: Date, pingTimeout: Duration) -> ConnectionEvaluation {
        switch connectionState {
        case .connecting:
            return handleConnectingState(now: now, pingTimeout: pingTimeout)
        case .connected:
            return handleConnectedState(now: now, pingTimeout: pingTimeout)
        default:
            return .ok
        }
    }

    func getPingTimeout() -> Duration {
        switch connectionState {
        case .connecting:
            let multiplier = timings.establishTimeoutMultiplier.saturatingPow(retryAttempt)
            let nextTimeout = timings.initialEstablishTimeout * Double(multiplier)

            if nextTimeout.isFinite, nextTimeout < timings.maxEstablishTimeout {
                return nextTimeout
            } else {
                return timings.maxEstablishTimeout
            }

        case .pendingStart, .connected, .waitingConnectivity, .stopped, .recovering:
            return timings.pingTimeout
        }
    }

    mutating func updateNetStats(newStats: WgStats, now: Date) {
        if newStats.bytesReceived > netStats.bytesReceived {
            lastSeenRx = now
        }

        if newStats.bytesSent > netStats.bytesSent {
            lastSeenTx = now
        }

        netStats = newStats
    }

    mutating func updatePingStats(sendResult: PingerSendResult, now: Date) {
        pingStats.requests.updateValue(now, forKey: sendResult.sequenceNumber)
        pingStats.lastRequestDate = now
    }

    mutating func setPingReplyReceived(_ sequenceNumber: UInt16, now: Date) -> Date? {
        guard let pingTimestamp = pingStats.requests.removeValue(forKey: sequenceNumber) else {
            return nil
        }

        pingStats.lastReplyDate = now
        timeoutReference = now

        return pingTimestamp
    }

    private func handleConnectingState(now: Date, pingTimeout: Duration) -> ConnectionEvaluation {
        if now.timeIntervalSince(timeoutReference) >= pingTimeout {
            return .pingTimeout
        }

        guard let lastRequestDate = pingStats.lastRequestDate else {
            return .sendInitialPing
        }

        if now.timeIntervalSince(lastRequestDate) >= timings.pingDelay {
            return .sendNextPing
        }

        return .ok
    }

    private func handleConnectedState(now: Date, pingTimeout: Duration) -> ConnectionEvaluation {
        if now.timeIntervalSince(timeoutReference) >= pingTimeout, !isHeartbeatSuspended {
            return .pingTimeout
        }

        guard let lastRequestDate = pingStats.lastRequestDate else {
            return .sendInitialPing
        }

        let timeSinceLastPing = now.timeIntervalSince(lastRequestDate)
        if let lastReplyDate = pingStats.lastReplyDate,
           lastReplyDate.timeIntervalSince(lastRequestDate) >= timings.heartbeatReplyTimeout,
           timeSinceLastPing >= timings.pingDelay, !isHeartbeatSuspended {
            return .retryHeartbeatPing
        }

        guard let lastSeenRx, let lastSeenTx else { return .ok }

        let rxTimeElapsed = now.timeIntervalSince(lastSeenRx)
        let txTimeElapsed = now.timeIntervalSince(lastSeenTx)

        if timeSinceLastPing >= timings.heartbeatPingInterval {
            // Send heartbeat if traffic is flowing.
            if rxTimeElapsed <= timings.trafficFlowTimeout || txTimeElapsed <= timings.trafficFlowTimeout {
                return .sendHeartbeatPing
            }

            if !isHeartbeatSuspended {
                return .suspendHeartbeat
            }
        }

        if timeSinceLastPing >= timings.pingDelay {
            if txTimeElapsed >= timings.trafficTimeout || rxTimeElapsed >= timings.trafficTimeout {
                return .trafficTimeout
            }

            if lastSeenTx > lastSeenRx, rxTimeElapsed >= timings.inboundTrafficTimeout {
                return .inboundTrafficTimeout
            }
        }

        return .ok
    }

    mutating func reset(resetRetryAttempts: Bool) {
        netStats = WgStats()
        lastSeenRx = nil
        lastSeenTx = nil
        pingStats = PingStats()
        isHeartbeatSuspended = false

        if resetRetryAttempts {
            retryAttempt = 0
        }
    }
}
