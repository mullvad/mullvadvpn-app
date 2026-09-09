// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

extension FileCache {
    public actor BridgeExecutor<T: Sendable> {
        public nonisolated var unownedExecutor: UnownedSerialExecutor {
        queue.asUnownedSerialExecutor()
    }

        // must only be accessed after semaphore has signalled
        nonisolated(unsafe) private var result: T? = nil
        // must only be accessed after semaphore has signalled
        nonisolated(unsafe) private var error: Error? = nil

        private let queue: DispatchSerialQueue = DispatchSerialQueue(label: "com.mullvad.vpn.bridge.executor")

        // The DispatchSemaphore ensures that @unchecked Sendable is safe.
        @available(*, noasync)
        public static nonisolated func run(
            _ closure: @escaping @Sendable () async throws -> T
        ) throws -> T {
            let bridge = BridgeExecutor()
            let semaphore = DispatchSemaphore(value: 0)


            bridge.queue.async {
                Task {
                    await bridge.execute(closure)
                    semaphore.signal()
                }
            }

            semaphore.wait()
            if let err = bridge.error {
                throw err
            }
            guard let result = bridge.result else {
                throw fatalError("Semaphore signaled without a result")
            }
            return result
        }

        //
        private func execute(_ closure: @Sendable () async throws -> T) async {
            do {
               result = try await closure()
            } catch {
                self.error = error
            }

        }
    }

    /// Wraps asynchronous tasks and runs them synchronously. Used in synchronous shims
    /// in FileCache and related files. Can be removed once those shims are removed.
    ///
    /// This is at best a forced solution to get what we want while migrating call sites
    /// to asynch, mainly on the Rust side.

    /// - Note:
    /// FileCache uses a DispatchSerialQueue as its custom actor executor, so calls to the
    /// shims happen on a GCD thread rather than a cooperative thread. This means it should
    /// be safe from deadlocks. Information and discussion here:
    /// https://forums.swift.org/t/using-async-functions-from-synchronous-functions-and-breaking-all-the-rules/59782/23
    ///
    /// - Warning:
    /// Calling this from anything backed by an the default executor, eg. MainActor, will
    /// very likely lead to a deadlock.
    private final class SynchRunner<T: Sendable>: @unchecked Sendable {
        private var result: Result<T, any Error>?

        // The DispatchSemaphore ensures that @unchecked Sendable is safe.
        @available(*, noasync)
        public static nonisolated func run(
            _ closure: @escaping @Sendable () async throws -> T
        ) throws -> T {
            let runner = SynchRunner<T>()
            let semaphore = DispatchSemaphore(value: 0)

            Task.detached(priority: Task.currentPriority) {
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
}
