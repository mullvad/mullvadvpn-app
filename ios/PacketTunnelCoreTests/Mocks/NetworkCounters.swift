//
//  NetworkCounters.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 16/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A type capable of receiving and updating network counters.
protocol NetworkStatsReporting {
    /// Increment number of bytes sent.
    func reportBytesSent(_ byteCount: UInt64)

    /// Increment number of bytes received.
    func reportBytesReceived(_ byteCount: UInt64)
}

/// A type providing network statistics.
protocol NetworkStatsProviding: Sendable {
    /// Returns number of bytes sent.
    var bytesSent: UInt64 { get }

    /// Returns number of bytes received.
    var bytesReceived: UInt64 { get }
}

/// Class that holds network statistics (bytes sent and received) for a simulated network adapter.
final class NetworkCounters: NetworkStatsProviding, NetworkStatsReporting, Sendable {
    private let stateLock = NSLock()
    nonisolated(unsafe) private var _bytesSent: UInt64 = 0
    nonisolated(unsafe) private var _bytesReceived: UInt64 = 0

    var bytesSent: UInt64 {
        stateLock.withLock { _bytesSent }
    }

    var bytesReceived: UInt64 {
        stateLock.withLock { _bytesReceived }
    }

    func reportBytesSent(_ byteCount: UInt64) {
        stateLock.withLock {
            _bytesSent += byteCount
        }
    }

    func reportBytesReceived(_ byteCount: UInt64) {
        stateLock.withLock {
            _bytesReceived += byteCount
        }
    }
}
