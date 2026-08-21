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

/// Multicasts `ObservedState` to any number of `AsyncStream` observers.
///
/// Deliberately not `Sendable`: an instance is owned by a single actor and must only be used
/// from its isolation, which is what serializes access. `send(_:)` is synchronous so it can be
/// called from a `didSet`, and delivery order matches the order states were produced.
///
/// A consumer that stops iterating early leaves a terminated continuation behind; yielding to
/// it is a no-op and it is pruned on the next `send`.
public struct ObservedStateBroadcaster: ~Copyable {
    private var continuations: [UUID: AsyncStream<ObservedState>.Continuation] = [:]
    private var isFinished = false

    deinit {
        finish()
    }

    /// Deliver `state` to every observer.
    ///
    /// Reaching `.disconnected` ends observation: all streams finish and later ones finish
    /// immediately.
    public mutating func send(_ state: ObservedState) {
        for (id, continuation) in continuations {
            if case .terminated = continuation.yield(state) {
                continuations.removeValue(forKey: id)
            }
        }

        if case .disconnected = state {
            isFinished = true
            finish()
        }
    }

    /// A stream that replays `currentState` and then receives every subsequent state.
    /// Finishes immediately if observation has already ended.
    public mutating func makeStream(replaying currentState: ObservedState) -> AsyncStream<ObservedState> {
        AsyncStream(bufferingPolicy: .unbounded) { continuation in
            continuation.yield(currentState)

            guard !isFinished else {
                continuation.finish()
                return
            }

            continuations[UUID()] = continuation
        }
    }

    /// End observation for all current and future observers.
    func finish() {
        for continuation in self.continuations.values {
            continuation.finish()
        }
    }
}
