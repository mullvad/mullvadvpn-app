//
//  PingerMock.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 16/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network
@testable import PacketTunnelCore

/// Ping client mock that can be used to simulate network transmission errors and delays.
class PingerMock: PingerProtocol {
    typealias OutcomeDecider = (IPv4Address, UInt16) -> Outcome

    private let decideOutcome: OutcomeDecider
    private let networkStatsReporting: NetworkStatsReporting
    private let stateLock = NSLock()
    private var state = State()

    var onReply: ((PingerReply) -> Void)? {
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

    func openSocket(bindTo interfaceName: String?) throws {
        stateLock.withLock {
            state.isSocketOpen = true
        }
    }

    func closeSocket() {
        stateLock.withLock {
            state.isSocketOpen = false
        }
    }

    func send(to address: IPv4Address) throws -> PingerSendResult {
        // Used for simulation. In reality can be any number.
        // But for realism it is: IPv4 header (20 bytes) + ICMP header (8 bytes)
        let icmpPacketSize: UInt = 28

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

        return PingerSendResult(sequenceNumber: nextSequenceId, bytesSent: icmpPacketSize)
    }

    // MARK: - Types

    /// Internal state
    private struct State {
        var sequenceId: UInt16 = 0
        var isSocketOpen = false
        var onReply: ((PingerReply) -> Void)?

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
