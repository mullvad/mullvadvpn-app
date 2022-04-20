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
    private let stateLock = NSLock()

    private var executionBlock: ((AsyncBlockOperation) -> Void)?
    private var cancellationBlocks: [() -> Void] = []

    init(block: @escaping (AsyncBlockOperation) -> Void) {
        executionBlock = block
    }

    override func main() {
        stateLock.lock()
        let block = executionBlock
        executionBlock = nil
        stateLock.unlock()

        block?(self)
    }

    override func finish() {
        stateLock.lock()
        cancellationBlocks.removeAll()
        executionBlock = nil
        stateLock.unlock()

        super.finish()
    }

    override func cancel() {
        super.cancel()

        stateLock.lock()
        let blocks = cancellationBlocks
        cancellationBlocks.removeAll()
        stateLock.unlock()

        for block in blocks {
            block()
        }
    }

    func addCancellationBlock(_ block: @escaping () -> Void) {
        stateLock.lock()
        if isCancelled {
            stateLock.unlock()
            block()
        } else {
            cancellationBlocks.append(block)
            stateLock.unlock()
        }
    }
}

