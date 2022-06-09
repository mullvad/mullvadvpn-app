//
//  TransformOperation.swift
//  AsyncOperationQueueTest
//
//  Created by pronebird on 31/05/2022.
//

import Foundation

final class TransformOperation<Input, Output, Failure: Error>:
    ResultOperation<Output, Failure>,
    InputOperation
{
    typealias ExecutionBlock = (Input, TransformOperation<Input, Output, Failure>) -> Void
    typealias ThrowingExecutionBlock = (Input) throws -> Output

    typealias InputBlock = () -> Input?

    private let nslock = NSLock()

    private(set) var input: Input? {
        get {
            nslock.lock()
            defer { nslock.unlock() }
            return _input
        }
        set {
            nslock.lock()
            _input = newValue
            nslock.unlock()
        }
    }

    private var _input: Input?
    private var inputBlock: InputBlock?

    private var executionBlock: ExecutionBlock?
    private var cancellationBlocks: [() -> Void] = []

    init(
        dispatchQueue: DispatchQueue? = nil,
        input: Input? = nil,
        block: ExecutionBlock? = nil
    )
    {
        _input = input
        executionBlock = block

        super.init(dispatchQueue: dispatchQueue)
    }

    init(
        dispatchQueue: DispatchQueue? = nil,
        input: Input? = nil,
        throwingBlock: @escaping ThrowingExecutionBlock
    )
    {
        _input = input
        executionBlock = Self.wrapThrowingBlock(throwingBlock)

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        let inputValue = inputBlock?()

        input = inputValue

        guard let inputValue = inputValue, let executionBlock = executionBlock else {
            finish(completion: .cancelled)
            return
        }

        executionBlock(inputValue, self)
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

    // MARK: - Block handlers

    func setExecutionBlock(_ block: @escaping ExecutionBlock) {
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

    // MARK: - Input injection

    func setInputBlock(_ block: @escaping () -> Input?) {
        dispatchQueue.async {
            self.inputBlock = block
        }
    }

    private class func wrapThrowingBlock(_ executionBlock: @escaping ThrowingExecutionBlock) -> ExecutionBlock {
        return { input, operation in
            do {
                let value = try executionBlock(input)

                operation.finish(completion: .success(value))
            } catch {
                let castedError = error as! Failure

                operation.finish(completion: .failure(castedError))
            }
        }
    }
}

