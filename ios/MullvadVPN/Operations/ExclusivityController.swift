//
//  ExclusivityController.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

class ExclusivityController<Category> where Category: Hashable {
    private let operationQueue: OperationQueue
    private let lock = NSRecursiveLock()

    private var operations: [Category: [Operation]] = [:]
    private var observers: [Operation: NSObjectProtocol] = [:]

    init(operationQueue: OperationQueue) {
        self.operationQueue = operationQueue
    }

    func addOperation(_ operation: Operation, categories: [Category]) {
        addOperations([operation], categories: categories)
    }

    func addOperations(_ operations: [Operation], categories: [Category]) {
        lock.withCriticalBlock {
            for operation in operations {
                for category in categories {
                    addDependencies(operation: operation, category: category)
                }

                observers[operation] = operation.observe(\.isFinished, options: [.initial, .new]) { [weak self] (op, change) in
                    if let isFinished = change.newValue, isFinished {
                        self?.operationDidFinish(op, categories: categories)
                    }
                }
            }

            operationQueue.addOperations(operations, waitUntilFinished: false)
        }
    }

    private func addDependencies(operation: Operation, category: Category) {
        var exclusiveOperations = self.operations[category] ?? []

        if let dependency = exclusiveOperations.last, !operation.dependencies.contains(dependency) {
            operation.addDependency(dependency)
        }

        exclusiveOperations.append(operation)
        self.operations[category] = exclusiveOperations
    }

    private func operationDidFinish(_ operation: Operation, categories: [Category]) {
        lock.withCriticalBlock {
            for category in categories {
                var exclusiveOperations = self.operations[category] ?? []

                exclusiveOperations.removeAll { (storedOperation) -> Bool in
                    return operation == storedOperation
                }

                self.operations[category] = exclusiveOperations
            }
            self.observers.removeValue(forKey: operation)
        }
    }
}
