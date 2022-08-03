//
//  ResultBlockOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 12/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

class ResultBlockOperation<Success, Failure: Error>: ResultOperation<Success, Failure> {
    typealias ExecutionBlock = (ResultBlockOperation<Success, Failure>) -> Void
    typealias ThrowingExecutionBlock = () throws -> Success

    private var executionBlock: ExecutionBlock?
    private var cancellationBlocks: [() -> Void] = []

    convenience init(
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

    convenience init(
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

    init(
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

    override func main() {
        let block = executionBlock
        executionBlock = nil

        block?(self)
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

    func setExecutionBlock(_ block: @escaping (ResultBlockOperation<Success, Failure>) -> Void) {
        dispatchQueue.async {
            assert(!self.isExecuting && !self.isFinished)
            self.executionBlock = block
        }
    }

    func setExecutionBlock(_ block: @escaping ThrowingExecutionBlock) {
        setExecutionBlock(Self.wrapThrowingBlock(block))
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

    private class func wrapThrowingBlock(_ executionBlock: @escaping ThrowingExecutionBlock)
        -> ExecutionBlock
    {
        return { operation in
            do {
                let value = try executionBlock()

                operation.finish(completion: .success(value))
            } catch {
                let castedError = error as! Failure

                operation.finish(completion: .failure(castedError))
            }
        }
    }
}
