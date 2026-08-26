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

/// Bridges a `Cancellable`-returning, completion-handler-based call (the shape used throughout the REST proxies)
/// into Swift concurrency, honoring `Task` cancellation by cancelling the underlying request.
public func withRESTContinuation<T>(
    _ operation: (@escaping @Sendable (Result<T, Error>) -> Void) -> Cancellable
) async throws -> T {
    let box = CancellableBox()
    return try await withTaskCancellationHandler {
        try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<T, Error>) in
            box.set(
                operation { result in
                    continuation.resume(with: result)
                })
        }
    } onCancel: {
        box.cancel()
    }
}

/// Thread-safe holder that cancels its `Cancellable` immediately if cancellation is requested before it's set.
private final class CancellableBox: @unchecked Sendable {
    private let lock = NSLock()
    private var cancellable: Cancellable?
    private var isCancelled = false

    func set(_ cancellable: Cancellable) {
        lock.withLock {
            if isCancelled {
                cancellable.cancel()
            } else {
                self.cancellable = cancellable
            }
        }
    }

    func cancel() {
        lock.withLock {
            isCancelled = true
            cancellable?.cancel()
        }
    }
}
