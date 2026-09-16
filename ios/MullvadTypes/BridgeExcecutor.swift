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
    private let queue: DispatchSerialQueue = DispatchSerialQueue(label: "BridgeExecutor")

    public nonisolated var unownedExecutor: UnownedSerialExecutor {
        queue.asUnownedSerialExecutor()
    }

    private init() {}

    @available(*, noasync)
    public nonisolated func run<T: Sendable>(
        _ closure: @escaping @Sendable () async throws -> T
    ) throws -> T {
        let box = ResultBox<T>()
        let semaphore = DispatchSemaphore(value: 0)

        Task { @BridgeExecutor in
            do {
                box.value = .success(try await closure())
            } catch {
                box.value = .failure(error)
            }
            semaphore.signal()
        }
        semaphore.wait()

        guard let result = box.value else { fatalError("Semaphore signalled without a result") }
        return try result.get()
    }
}

// The DispatchSemaphore in BridgeExecutor ensures that @unchecked Sendable is safe.
private final class ResultBox<T: Sendable>: @unchecked Sendable {
    var value: Result<T, Error>?
}
