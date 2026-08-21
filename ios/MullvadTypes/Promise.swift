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

// This object can be used like an async semaphore with exactly 1 writer. It
// allows the waiter to wait to `receive()` from another operation
// asynchronously. It is important not to forget to call `send`, otherwise this
// operation will block indefinitely.
public struct OneshotChannel: Sendable {
    private var continuation: AsyncStream<Void>.Continuation?
    private var stream: AsyncStream<Void>

    public init() {
        var ownedContinuation: AsyncStream<Void>.Continuation?
        stream = AsyncStream { continuation in
            ownedContinuation = continuation
        }
        self.continuation = ownedContinuation
    }

    public func send() {
        continuation?.yield()
        continuation?.finish()
    }

    public func receive() async {
        for await _ in stream {
            return
        }
    }
}
