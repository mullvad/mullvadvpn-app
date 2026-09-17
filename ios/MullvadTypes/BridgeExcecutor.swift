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
    /// Wraps asynchronous tasks and runs them synchronously. Used in synchronous shims
    /// in FileCache and related files. Can be removed once those shims are removed.
    public actor BridgeExecutor<T: Sendable> {
        public nonisolated var unownedExecutor: UnownedSerialExecutor {
            queue.asUnownedSerialExecutor()
        }

        // Must only be accessed after semaphore has signalled.
        nonisolated(unsafe) private var result: T? = nil
        // Must only be accessed after semaphore has signalled.
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

            if let error = bridge.error {
                throw error
            }

            guard let result = bridge.result else {
                fatalError("Semaphore signaled without a result")
            }

            return result
        }

        private func execute(_ closure: @Sendable () async throws -> T) async {
            do {
                result = try await closure()
            } catch {
                self.error = error
            }
        }
    }
}
