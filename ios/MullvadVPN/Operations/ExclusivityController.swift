//
//  ExclusivityController.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

class ExclusivityController: NSObject {
    private let lock = NSLock()
    private var operations: [String: [Operation]] = [:]
    private var categoriesByOperation: [Operation: [String]] = [:]

    func addOperation(_ operation: Operation, categories: [String]) {
        lock.withCriticalBlock {
            categories.forEach { category in
                addOperation(operation, category: category)
            }

            addObserverIfNeeded(operation: operation, categories: categories)
        }
    }

    func removeOperation(_ operation: Operation, categories: [String]) {
        lock.withCriticalBlock {
            categories.forEach { category in
                removeOperation(operation, category: category)
            }

            removeObserverIfNeeded(operation: operation, categories: categories)
        }
    }

    override func observeValue(forKeyPath keyPath: String?, of object: Any?, change: [NSKeyValueChangeKey : Any]?, context: UnsafeMutableRawPointer?) {
        if let operation = object as? Operation, keyPath == "isFinished" {
            operationDidFinish(operation)
        } else {
            super.observeValue(forKeyPath: keyPath, of: object, change: change, context: context)
        }
    }

    // MARK: - Private

    private func addOperation(_ operation: Operation, category: String) {
        var operationsWithThisCategory = operations[category] ?? []

        if let last = operationsWithThisCategory.last {
            operation.addDependency(last)
        }

        operationsWithThisCategory.append(operation)

        operations[category] = operationsWithThisCategory
    }

    private func removeOperation(_ operation: Operation, category: String) {
        guard var operationsWithThisCategory = operations[category],
              let index = operationsWithThisCategory.firstIndex(of: operation) else { return }

        operationsWithThisCategory.remove(at: index)

        if operationsWithThisCategory.isEmpty {
            operations.removeValue(forKey: category)
        } else {
            operations[category] = operationsWithThisCategory
        }
    }

    private func addObserverIfNeeded(operation: Operation, categories: [String]) {
        let existingCategories = categoriesByOperation[operation] ?? []
        let newCategories = existingCategories + categories

        if existingCategories.isEmpty && !newCategories.isEmpty {
            operation.addObserver(self, forKeyPath: "isFinished", options: .new, context: nil)
        }

        if !newCategories.isEmpty {
            categoriesByOperation[operation] = newCategories
        }
    }

    private func removeObserverIfNeeded(operation: Operation, categories: [String]) {
        guard var newCategories = categoriesByOperation[operation] else { return }

        newCategories.removeAll { s in
            categories.contains(s)
        }

        if newCategories.isEmpty {
            operation.removeObserver(self, forKeyPath: "isFinished", context: nil)

            categoriesByOperation.removeValue(forKey: operation)
        } else {
            categoriesByOperation[operation] = newCategories
        }
    }

    private func operationDidFinish(_ operation: Operation) {
        lock.withCriticalBlock {
            let operationCategories = categoriesByOperation[operation] ?? []

            removeObserverIfNeeded(operation: operation, categories: operationCategories)

            operationCategories.forEach { category in
                removeOperation(operation, category: category)
            }
        }
    }
}
