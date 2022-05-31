//
//  TransformOperation.swift
//  AsyncOperationQueueTest
//
//  Created by pronebird on 31/05/2022.
//

import Foundation

final class TransformOperation<Input, Output, Failure: Error>: ResultOperation<Output, Failure> {
    typealias ExecutionBlock = ((Input, TransformOperation<Input, Output, Failure>) -> Void)
    typealias ThrowingExecutionBlock = ((Input) throws -> Output)

    private var input: Input?
    private var executionBlock: ExecutionBlock?
    private var configurationBlocks: [() -> Void] = []
    private var cancellationBlocks: [() -> Void] = []

    init(
        dispatchQueue: DispatchQueue? = nil,
        input: Input? = nil,
        block: ExecutionBlock? = nil
    )
    {
        self.input = input
        self.executionBlock = block

        super.init(dispatchQueue: dispatchQueue)
    }

    convenience init(
        dispatchQueue: DispatchQueue?,
        input: Input? = nil,
        block: @escaping ThrowingExecutionBlock
    )
    {
        self.init(
            dispatchQueue: dispatchQueue,
            input: input,
            block: Self.wrapThrowingBlock(block)
        )
    }

    override func main() {
        for configurationBlock in configurationBlocks {
            configurationBlock()
        }

        configurationBlocks.removeAll()

        guard let input = input, let executionBlock = executionBlock else {
            finish(completion: .cancelled)
            return
        }

        executionBlock(input, self)
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

    func inject<T>(from dependency: T) where T: OutputOperation, T.Output == Input {
        inject(from: dependency, via: { $0 })
    }

    func inject<T>(from dependency: T, via block: @escaping (T.Output) -> Input) where T: OutputOperation {
        dispatchQueue.async {
            self.configurationBlocks.append { [weak self] in
                guard let self = self else { return }

                if let output = dependency.output {
                    self.input = block(output)
                }
            }

        }
        addDependency(dependency)
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

