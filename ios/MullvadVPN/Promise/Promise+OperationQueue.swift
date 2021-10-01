//
//  Promise+OperationQueue.swift
//  Promise+OperationQueue
//
//  Created by pronebird on 02/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Promise {
    /// Returns a promise that adds a mutually exclusive operation that finishes along with the upstream.
    func run(on operationQueue: OperationQueue, categories: [String] = []) -> Promise<Value> {
        return Promise(parent: self) { resolver in
            let operation = AsyncBlockOperation { operation in
                let completionQueue = operationQueue.underlyingQueue

                if operation.isCancelled {
                    resolver.resolve(completion: .cancelled, queue: completionQueue)
                    operation.finish()
                } else {
                    self.observe { completion in
                        resolver.resolve(completion: completion, queue: completionQueue)
                        operation.finish()
                    }
                }
            }

            resolver.setCancelHandler {
                operation.cancel()
            }

            if !categories.isEmpty {
                ExclusivityController.shared.addOperation(operation, categories: categories)
            }

            operationQueue.addOperation(operation)
        }
    }
}
