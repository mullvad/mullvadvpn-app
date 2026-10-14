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
import os

/// A `Clock` whose time only moves when a test moves it.
///
/// Sleepers suspend until `advance(by:)` passes their deadline, so code under test can use
/// production timings without a test ever waiting in real time.
///
/// `advance(by:)` must not run before the code under test has actually
/// suspended on `sleep`. Anchor each call on an awaited observation
/// that provably follows the sleep.
final class TestClock: Clock {
    struct Instant: InstantProtocol {
        var offset: Duration

        func advanced(by duration: Duration) -> Instant {
            Instant(offset: offset + duration)
        }

        func duration(to other: Instant) -> Duration {
            other.offset - offset
        }

        static func < (lhs: Instant, rhs: Instant) -> Bool {
            lhs.offset < rhs.offset
        }
    }

    private struct Sleeper {
        let deadline: Instant
        let continuation: CheckedContinuation<Void, Error>
    }

    private enum Disposition {
        case cancelled, elapsed
        case suspended([SleeperWaiter])
    }

    private struct SleeperWaiter {
        let count: Int
        let continuation: CheckedContinuation<Void, Never>
    }

    private struct State {
        var currentInstant = Instant(offset: .zero)
        var sleepers: [Int: Sleeper] = [:]
        var cancelledIDs: Set<Int> = []
        var lastID = 0
        var sleeperWaiters: [SleeperWaiter] = []

        /// Removes and returns the waiters satisfied by the current number of sleepers.
        mutating func takeSatisfiedWaiters() -> [SleeperWaiter] {
            let satisfied = sleeperWaiters.filter { $0.count <= sleepers.count }
            sleeperWaiters.removeAll { $0.count <= sleepers.count }
            return satisfied
        }
    }

    private let state = OSAllocatedUnfairLock(initialState: State())

    var now: Instant {
        state.withLock { $0.currentInstant }
    }

    var minimumResolution: Duration {
        .zero
    }

    /// Number of sleepers currently suspended. Lets a test assert that a timer was scheduled
    /// or cancelled without waiting for it.
    var sleeperCount: Int {
        state.withLock { $0.sleepers.count }
    }

    func sleep(until deadline: Instant, tolerance: Duration? = nil) async throws {
        let id = state.withLock { state in
            state.lastID += 1
            return state.lastID
        }

        try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                let disposition = state.withLock { state -> Disposition in
                    // Cancellation may arrive before the continuation exists.
                    guard state.cancelledIDs.remove(id) == nil else { return .cancelled }
                    guard state.currentInstant < deadline else { return .elapsed }
                    state.sleepers[id] = Sleeper(deadline: deadline, continuation: continuation)
                    return .suspended(state.takeSatisfiedWaiters())
                }

                switch disposition {
                case .cancelled: continuation.resume(throwing: CancellationError())
                case .elapsed: continuation.resume()
                case let .suspended(waiters): waiters.forEach { $0.continuation.resume() }
                }
            }
        } onCancel: {
            let sleeper = state.withLock { state -> Sleeper? in
                guard let sleeper = state.sleepers.removeValue(forKey: id) else {
                    state.cancelledIDs.insert(id)
                    return nil
                }
                return sleeper
            }
            sleeper?.continuation.resume(throwing: CancellationError())
        }
    }

    /// Move time forward, waking every sleeper whose deadline is reached, in deadline order.
    ///
    /// Time stops at each deadline before moving on, so a periodic sleeper that reschedules
    /// itself is woken once per period rather than once in total.
    func advance(by duration: Duration) async {
        let target = state.withLock { $0.currentInstant.advanced(by: duration) }

        while let sleeper = takeNextSleeper(reaching: target) {
            sleeper.continuation.resume()
            await settle()
        }

        state.withLock { $0.currentInstant = target }
        await settle()
    }

    /// Wait until at least `count` sleepers are suspended.
    ///
    /// Code under test registers its next sleep asynchronously (a periodic timer only re-arms
    /// once the event it emitted has been handled), so `advance(by:)` needs the sleeper to exist
    /// before it can wake it. Resumed by the `sleep` call that brings the count up to `count`.
    func waitForSleepers(_ count: Int = 1) async {
        await withCheckedContinuation { continuation in
            let isSatisfied = state.withLock { state in
                guard state.sleepers.count < count else { return true }
                state.sleeperWaiters.append(SleeperWaiter(count: count, continuation: continuation))
                return false
            }
            if isSatisfied {
                continuation.resume()
            }
        }
    }

    /// Wake every sleeper, however far in the future, until none are left.
    func runToCompletion() async {
        while let sleeper = takeNextSleeper(reaching: nil) {
            sleeper.continuation.resume()
            await settle()
        }
    }

    /// Removes and returns the earliest sleeper at or before `limit`, moving time to its deadline.
    private func takeNextSleeper(reaching limit: Instant?) -> Sleeper? {
        state.withLock { state -> Sleeper? in
            guard let (id, sleeper) = state.sleepers.min(by: { $0.value.deadline < $1.value.deadline })
            else { return nil }

            if let limit, sleeper.deadline > limit { return nil }

            state.currentInstant = sleeper.deadline
            state.sleepers.removeValue(forKey: id)
            return sleeper
        }
    }

    /// Yield to the executor to hopefully allow other tasks to register more sleep calls.
    private func settle() async {
        for _ in 0..<20 {
            await Task.yield()
        }
    }
}
