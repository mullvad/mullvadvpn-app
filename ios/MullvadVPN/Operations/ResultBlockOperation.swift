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

    private var executionBlock: ExecutionBlock?
    private var cancellationBlocks: [() -> Void] = []

    convenience init(dispatchQueue: DispatchQueue?, executionBlock: @escaping ExecutionBlock) {
        self.init(
            dispatchQueue: dispatchQueue,
            executionBlock: executionBlock,
            completionQueue: nil,
            completionHandler: nil
        )
    }

    init(
        dispatchQueue: DispatchQueue?,
        executionBlock: @escaping ExecutionBlock,
        completionQueue: DispatchQueue?,
        completionHandler: CompletionHandler?
    )
    {
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

