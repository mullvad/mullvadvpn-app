//
//  AsyncBlockOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Asynchronous block operation
class AsyncBlockOperation: AsyncOperation {
    private var executionBlock: ((AsyncBlockOperation) -> Void)?
    private var cancellationBlocks: [() -> Void] = []

    override init(dispatchQueue: DispatchQueue? = nil) {
        super.init(dispatchQueue: dispatchQueue)
    }

    init(dispatchQueue: DispatchQueue? = nil, block: @escaping (AsyncBlockOperation) -> Void) {
        executionBlock = block
        super.init(dispatchQueue: dispatchQueue)
    }

    init(dispatchQueue: DispatchQueue? = nil, block: @escaping () -> Void) {
        executionBlock = { operation in
            block()
            operation.finish()
        }
        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        let block = executionBlock
        executionBlock = nil

        if let block = block {
            block(self)
        } else {
            finish()
        }
    }

    override func operationDidCancel() {
        let blocks = cancellationBlocks
        cancellationBlocks.removeAll()

        for block in blocks {
            block()
        }
    }

    override func operationDidFinish() {
        cancellationBlocks.removeAll()
        executionBlock = nil
    }

    func setExecutionBlock(_ block: @escaping (AsyncBlockOperation) -> Void) {
        dispatchQueue.async {
            assert(!self.isExecuting && !self.isFinished)
            self.executionBlock = block
        }
    }

    func setExecutionBlock(_ block: @escaping () -> Void) {
        setExecutionBlock { operation in
            block()
            operation.finish()
        }
    }

    func addCancellationBlock(_ block: @escaping () -> Void) {
        dispatchQueue.async {
            if self.isCancelled {
                block()
            } else {
                self.cancellationBlocks.append(block)
            }
        }
    }
}

