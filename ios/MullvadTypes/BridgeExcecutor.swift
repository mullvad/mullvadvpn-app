// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

/// Wraps asynchronous tasks and runs them synchronously. Used in synchronous shims
/// in FileCache and related files. Can be removed once those shims are removed.
@globalActor
public actor BridgeExecutor {
    public static let shared = BridgeExecutor()

    public nonisolated var unownedExecutor: UnownedSerialExecutor {
        queue.asUnownedSerialExecutor()
    }

    private let queue: DispatchSerialQueue = DispatchSerialQueue(label: "BridgeExecutor")

    private init() {}

    @available(*, noasync)
    public nonisolated func run<T: Sendable>(
        _ closure: @escaping @Sendable () async throws -> T
    ) throws -> T {
        let box = ResultBox<T>()
        let semaphore = DispatchSemaphore(value: 0)

        Task { @BridgeExecutor in
            do {
                box.result = try await closure()
            } catch {
                box.error = error
            }
            semaphore.signal()
        }
        semaphore.wait()

        if let error = box.error {
            throw error
        }

        guard let result = box.result else {
            fatalError("Semaphore signaled without a result")
        }

        return result
    }
}

// The DispatchSemaphore in BridgeExecutor ensures that @unchecked Sendable is safe.
private final class ResultBox<T: Sendable>: @unchecked Sendable {
    /// Must only be accessed after semaphore has signalled.
    var result: T?
    /// Must only be accessed after semaphore has signalled.
    var error: Error?
}
