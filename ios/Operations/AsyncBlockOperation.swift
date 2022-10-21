//
//  AsyncBlockOperation.swift
//  Operations
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Asynchronous block operation
public class AsyncBlockOperation: AsyncOperation {
    private var executionBlock: ((AsyncBlockOperation) -> Void)?
    private var cancellationBlocks: [() -> Void] = []

    override public init(dispatchQueue: DispatchQueue? = nil) {
        super.init(dispatchQueue: dispatchQueue)
    }

    public init(
        dispatchQueue: DispatchQueue? = nil,
        block: @escaping (AsyncBlockOperation) -> Void
    ) {
        executionBlock = block
        super.init(dispatchQueue: dispatchQueue)
    }

    public init(dispatchQueue: DispatchQueue? = nil, block: @escaping () -> Void) {
        executionBlock = { operation in
            block()
            operation.finish()
        }
        super.init(dispatchQueue: dispatchQueue)
    }

    override public func main() {
        let block = executionBlock
        executionBlock = nil

        if let block = block {
            block(self)
        } else {
            finish()
        }
    }

    override public func operationDidCancel() {
        let blocks = cancellationBlocks
        cancellationBlocks.removeAll()

        for block in blocks {
            block()
        }
    }

    override public func operationDidFinish() {
        cancellationBlocks.removeAll()
        executionBlock = nil
    }

    public func setExecutionBlock(_ block: @escaping (AsyncBlockOperation) -> Void) {
        dispatchQueue.async {
            assert(!self.isExecuting && !self.isFinished)
            self.executionBlock = block
        }
    }

    public func setExecutionBlock(_ block: @escaping () -> Void) {
        setExecutionBlock { operation in
            block()
            operation.finish()
        }
    }

    public func addCancellationBlock(_ block: @escaping () -> Void) {
        dispatchQueue.async {
            if self.isCancelled {
                block()
            } else {
                self.cancellationBlocks.append(block)
            }
        }
    }
}
