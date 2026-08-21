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

public final class AsyncOperationQueue: OperationQueue, @unchecked Sendable {
    override public func addOperation(_ operation: Operation) {
        if let operation = operation as? AsyncOperation {
            let categories = operation.conditions
                .filter { condition in
                    condition.isMutuallyExclusive
                }
                .map { condition in
                    condition.name
                }

            if !categories.isEmpty {
                ExclusivityManager.shared.addOperation(operation, categories: Set(categories))
            }

            super.addOperation(operation)

            operation.didEnqueue()
        } else {
            super.addOperation(operation)
        }
    }

    override public func addOperations(_ operations: [Operation], waitUntilFinished wait: Bool) {
        for operation in operations {
            addOperation(operation)
        }

        if wait {
            for operation in operations {
                operation.waitUntilFinished()
            }
        }
    }

    public static func makeSerial() -> AsyncOperationQueue {
        let queue = AsyncOperationQueue()
        queue.maxConcurrentOperationCount = 1
        return queue
    }
}

private final class ExclusivityManager: @unchecked Sendable {
    static let shared = ExclusivityManager()

    private var operationsByCategory = [String: [Operation]]()
    private let nslock = NSLock()

    private init() {}

    func addOperation(_ operation: AsyncOperation, categories: Set<String>) {
        nslock.withLock {
            for category in categories {
                var operations = operationsByCategory[category] ?? []

                if let lastOperation = operations.last {
                    operation.addDependency(lastOperation)
                }

                operations.append(operation)

                operationsByCategory[category] = operations

                operation.onFinish { [weak self] op, _ in
                    self?.removeOperation(op, categories: categories)
                }
            }
        }
    }

    private func removeOperation(_ operation: Operation, categories: Set<String>) {
        nslock.withLock {
            for category in categories {
                guard var operations = operationsByCategory[category] else {
                    continue
                }

                if let index = operations.firstIndex(of: operation) {
                    // Break the successor's back-pointer so finished operations can be
                    // released immediately. Without this, addDependency chains every op
                    // to its predecessor — under load, releasing the tail triggers a
                    // recursive deinit cascade that overflows the stack.
                    let nextIndex = operations.index(after: index)
                    if nextIndex < operations.endIndex {
                        operations[nextIndex].removeDependency(operation)
                    }
                    operations.remove(at: index)
                }

                if operations.isEmpty {
                    operationsByCategory.removeValue(forKey: category)
                } else {
                    operationsByCategory[category] = operations
                }
            }
        }
    }
}
