//
//  ResultBlockOperation.swift
//  Operations
//
//  Created by pronebird on 12/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public final class ResultBlockOperation<Success>: ResultOperation<Success> {
    public typealias ExecutionBlock = (ResultBlockOperation<Success>) -> Void
    public typealias ThrowingExecutionBlock = () throws -> Success

    private var executionBlock: ExecutionBlock?
    private var cancellationBlocks: [() -> Void] = []

    public convenience init(
        dispatchQueue: DispatchQueue? = nil,
        executionBlock: ExecutionBlock? = nil
    ) {
        self.init(
            dispatchQueue: dispatchQueue,
            executionBlock: executionBlock,
            completionQueue: nil,
            completionHandler: nil
        )
    }

    public convenience init(
        dispatchQueue: DispatchQueue? = nil,
        executionBlock: @escaping ThrowingExecutionBlock
    ) {
        self.init(
            dispatchQueue: dispatchQueue,
            executionBlock: Self.wrapThrowingBlock(executionBlock),
            completionQueue: nil,
            completionHandler: nil
        )
    }

    public init(
        dispatchQueue: DispatchQueue?,
        executionBlock: ExecutionBlock?,
        completionQueue: DispatchQueue?,
        completionHandler: CompletionHandler?
    ) {
        self.executionBlock = executionBlock

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: completionQueue,
            completionHandler: completionHandler
        )
    }

    override public func main() {
        let block = executionBlock
        executionBlock = nil

        block?(self)
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

    public func setExecutionBlock(_ block: @escaping ExecutionBlock) {
        dispatchQueue.async {
            assert(!self.isExecuting && !self.isFinished)
            self.executionBlock = block
        }
    }

    public func setExecutionBlock(_ block: @escaping ThrowingExecutionBlock) {
        setExecutionBlock(Self.wrapThrowingBlock(block))
    }

    public func addCancellationBlock(_ block: @escaping () -> Void) {
        dispatchQueue.async {
            if self.isCancelled, self.isExecuting {
                block()
            } else {
                self.cancellationBlocks.append(block)
            }
        }
    }

    private class func wrapThrowingBlock(_ executionBlock: @escaping ThrowingExecutionBlock)
        -> ExecutionBlock
    {
        return { operation in
            let result = Result { try executionBlock() }

            operation.finish(result: result)
        }
    }
}
