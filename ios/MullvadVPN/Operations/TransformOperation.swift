//
//  TransformOperation.swift
//  AsyncOperationQueueTest
//
//  Created by pronebird on 31/05/2022.
//

import Foundation

final class TransformOperation<Input, Output, Failure: Error>: ResultOperation<Output, Failure> {
    private var input: Input?

    typealias ExecutionBlock = ((Input, TransformOperation<Input, Output, Failure>) -> Void)

    private var executionBlock: ExecutionBlock?
    private var configurationBlocks: [() -> Void] = []
    private var cancellationBlocks: [() -> Void] = []

    init(dispatchQueue: DispatchQueue?, input: Input? = nil, block: @escaping ExecutionBlock) {
        self.input = input
        self.executionBlock = block

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        for configurationBlock in configurationBlocks {
            configurationBlock()
        }

        configurationBlocks.removeAll()

        guard let input = input else {
            finish(completion: .cancelled)
            return
        }

        executionBlock?(input, self)
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

}

