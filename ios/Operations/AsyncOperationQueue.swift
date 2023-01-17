//
//  AsyncOperationQueue.swift
//  Operations
//
//  Created by pronebird on 30/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public final class AsyncOperationQueue: OperationQueue {
    override public func addOperation(_ operation: Operation) {
        if let operation = operation as? AsyncOperation {
            let categories = operation.conditions
                .filter { condition in
                    return condition.isMutuallyExclusive
                }
                .map { condition in
                    return condition.name
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

private final class ExclusivityManager {
    static let shared = ExclusivityManager()

    private var operationsByCategory = [String: [Operation]]()
    private let nslock = NSLock()

    private init() {}

    func addOperation(_ operation: AsyncOperation, categories: Set<String>) {
        nslock.lock()
        defer { nslock.unlock() }

        for category in categories {
            var operations = operationsByCategory[category] ?? []

            if let lastOperation = operations.last {
                operation.addDependency(lastOperation)
            }

            operations.append(operation)

            operationsByCategory[category] = operations

            let blockObserver = OperationBlockObserver(didFinish: { [weak self] op, error in
                self?.removeOperation(op, categories: categories)
            })

            operation.addObserver(blockObserver)
        }
    }

    private func removeOperation(_ operation: Operation, categories: Set<String>) {
        nslock.lock()
        defer { nslock.unlock() }

        for category in categories {
            guard var operations = operationsByCategory[category] else {
                continue
            }

            if let index = operations.firstIndex(of: operation) {
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
