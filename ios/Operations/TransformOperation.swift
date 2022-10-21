//
//  TransformOperation.swift
//  Operations
//
//  Created by pronebird on 31/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public final class TransformOperation<Input, Output, Failure: Error>:
    ResultOperation<Output, Failure>,
    InputOperation
{
    public typealias ExecutionBlock = (Input, TransformOperation<Input, Output, Failure>) -> Void
    public typealias ThrowingExecutionBlock = (Input) throws -> Output
    public typealias InputBlock = () -> Input?

    private let nslock = NSLock()

    public var input: Input? {
        return _input
    }

    private var __input: Input?
    private var _input: Input? {
        get {
            nslock.lock()
            defer { nslock.unlock() }
            return __input
        }
        set {
            nslock.lock()
            __input = newValue
            nslock.unlock()
        }
    }

    private var inputBlock: InputBlock?

    private var executionBlock: ExecutionBlock?
    private var cancellationBlocks: [() -> Void] = []

    public init(
        dispatchQueue: DispatchQueue? = nil,
        input: Input? = nil,
        block: ExecutionBlock? = nil
    ) {
        __input = input
        executionBlock = block

        super.init(dispatchQueue: dispatchQueue)
    }

    public init(
        dispatchQueue: DispatchQueue? = nil,
        input: Input? = nil,
        throwingBlock: @escaping ThrowingExecutionBlock
    ) {
        __input = input
        executionBlock = Self.wrapThrowingBlock(throwingBlock)

        super.init(dispatchQueue: dispatchQueue)
    }

    override public func main() {
        let inputValue = inputBlock?()

        _input = inputValue

        guard let inputValue = inputValue, let executionBlock = executionBlock else {
            finish(completion: .cancelled)
            return
        }

        executionBlock(inputValue, self)
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

    // MARK: - Block handlers

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
            if self.isCancelled {
                block()
            } else {
                self.cancellationBlocks.append(block)
            }
        }
    }

    // MARK: - Input injection

    public func setInputBlock(_ block: @escaping () -> Input?) {
        dispatchQueue.async {
            self.inputBlock = block
        }
    }

    private class func wrapThrowingBlock(_ executionBlock: @escaping ThrowingExecutionBlock)
        -> ExecutionBlock
    {
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
