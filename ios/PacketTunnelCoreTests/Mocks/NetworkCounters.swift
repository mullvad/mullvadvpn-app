// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

/// A type capable of receiving and updating network counters.
protocol NetworkStatsReporting {
    /// Increment number of bytes sent.
    func reportBytesSent(_ byteCount: UInt64)

    /// Increment number of bytes received.
    func reportBytesReceived(_ byteCount: UInt64)
}

/// A type providing network statistics.
protocol NetworkStatsProviding {
    /// Returns number of bytes sent.
    var bytesSent: UInt64 { get }

    /// Returns number of bytes received.
    var bytesReceived: UInt64 { get }
}

/// Class that holds network statistics (bytes sent and received) for a simulated network adapter.
final class NetworkCounters: NetworkStatsProviding, NetworkStatsReporting {
    private let stateLock = NSLock()
    private var _bytesSent: UInt64 = 0
    private var _bytesReceived: UInt64 = 0

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
