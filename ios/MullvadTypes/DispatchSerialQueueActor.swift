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

/// Actor whose executor is a `DispatchSerialQueue`.
///
/// Because isolated work runs on a GCD queue rather than the cooperative thread pool, synchronous
/// callers can run isolated work by blocking on that queue without needing a pool thread.
public protocol DispatchSerialQueueActor: Actor {
    nonisolated var queue: DispatchSerialQueue { get }
}

extension DispatchSerialQueueActor {
    /// Runs `body` isolated to this actor, blocking the calling thread until it completes.
    ///
    /// Never involves the cooperative thread pool, so it cannot deadlock a starved pool. It must
    /// not be called from the actor's own queue, and `body` must not call back into a blocking
    /// entry point of the same actor.
    @available(*, noasync, message: "Use the async API from asynchronous contexts")
    public nonisolated func runBlocking<T>(_ body: (isolated Self) throws -> T) rethrows -> T {
        try queue.sync {
            try assumeIsolated(body)
        }
    }
}
