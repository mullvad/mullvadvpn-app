//
//  PingerMock.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 16/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network
@testable import PacketTunnelCore

/// Ping client mock that can be used to simulate network transmission errors and delays.
final class PingerMock: PingerProtocol, @unchecked Sendable {
    typealias OutcomeDecider = (IPv4Address, UInt16) -> Outcome

    let decideOutcome: OutcomeDecider
    let networkStatsReporting: NetworkStatsReporting
    let stateLock = NSLock()
    var state = State()

    var onReply: (@Sendable (PingerReply) -> Void)? {
        get {
            stateLock.withLock { state.onReply }
        }
        set {
            stateLock.withLock { state.onReply = newValue }
        }
    }

    init(networkStatsReporting: NetworkStatsReporting, decideOutcome: @escaping OutcomeDecider) {
        self.networkStatsReporting = networkStatsReporting
        self.decideOutcome = decideOutcome
    }

    func startPinging(destAddress: IPv4Address) {
        stateLock.withLock {
            state.destAddress = destAddress
            state.isSocketOpen = true
        }
    }

    func stopPinging() {
        stateLock.withLock {
            state.isSocketOpen = false
        }
    }

    func send() throws -> PingerSendResult {
        // Used for simulation. In reality can be any number.
        // But for realism it is: IPv4 header (20 bytes) + ICMP header (8 bytes)
        let icmpPacketSize: UInt = 28

        guard let address = state.destAddress else {
            fatalError("Address somehow not set when sending ping")
        }

        let nextSequenceId = try stateLock.withLock {
            guard state.isSocketOpen else { throw POSIXError(.ENOTCONN) }

            return state.incrementSequenceId()
        }

        switch decideOutcome(address, nextSequenceId) {
        case let .sendReply(reply, delay):
            DispatchQueue.main.asyncAfter(wallDeadline: .now() + delay) { [weak self] in
                guard let self else { return }

                networkStatsReporting.reportBytesReceived(UInt64(icmpPacketSize))

                switch reply {
                case .normal:
                    onReply?(.success(address, nextSequenceId))
                case .malformed:
                    onReply?(.parseError(ParseError()))
                }
            }

        case .ignore:
            break

        case .sendFailure:
            throw POSIXError(.ECONNREFUSED)
        }

        networkStatsReporting.reportBytesSent(UInt64(icmpPacketSize))

        return PingerSendResult(sequenceNumber: nextSequenceId)
    }

    // MARK: - Types

    /// Internal state
    struct State {
        var sequenceId: UInt16 = 0
        var isSocketOpen = false
        var onReply: (@Sendable (PingerReply) -> Void)?
        var destAddress: IPv4Address?

        mutating func incrementSequenceId() -> UInt16 {
            sequenceId += 1
            return sequenceId
        }
    }

    /// Simulated ICMP reply.
    enum Reply {
        /// Simulate normal ping reply.
        case normal

        /// Simulate malformed ping reply.
        case malformed
    }

    /// The outcome of ping request simulation.
    enum Outcome {
        /// Simulate ping reply transmission.
        case sendReply(reply: Reply = .normal, afterDelay: Duration = .milliseconds(100))

        /// Simulate packet that was lost or left unanswered.
        case ignore

        /// Simulate failure to send ICMP packet (i.e `sendto()` error).
        case sendFailure
    }

    struct ParseError: LocalizedError {
        var errorDescription: String? {
            return "ICMP response parse error"
        }
    }
}
