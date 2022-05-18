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

    init(dispatchQueue: DispatchQueue?, block: @escaping (AsyncBlockOperation) -> Void) {
        executionBlock = block
        super.init(dispatchQueue: dispatchQueue)
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

