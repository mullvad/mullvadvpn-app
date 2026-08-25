// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

// Wraps asynchronous tasks and runs them synchronously. Mainly used in synchronous
// shims in FileCache and related files. Can be removed once those shims are removed.
public final class SynchRunner<T: Sendable>: @unchecked Sendable {
    private var result: Result<T, any Error>?

    // The DispatchSemaphore ensures that @unchecked Sendable is safe.
    public static nonisolated func run(
        _ closure: @escaping @Sendable () async throws -> T
    ) throws -> T {
        let runner = SynchRunner<T>()
        let semaphore = DispatchSemaphore(value: 0)
        let priority = TaskPriority(rawValue: UInt8(qos_class_self().rawValue))

        Task.detached(priority: priority) {
            do {
                runner.result = .success(try await closure())
            } catch {
                runner.result = .failure(error)
            }
            semaphore.signal()
        }
        semaphore.wait()

        // Result is always set, so force-unwrap is safe.
        switch runner.result! {
        case let .success(value):
            return value
        case let .failure(error):
            throw error
        }
    }
}
