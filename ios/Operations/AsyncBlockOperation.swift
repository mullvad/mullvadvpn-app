//
//  AsyncBlockOperation.swift
//  Operations
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import protocol MullvadTypes.Cancellable

/// Asynchronous block operation
public class AsyncBlockOperation: AsyncOperation {
    public typealias ExecutionBlock = (AsyncBlockOperation) -> Void

    private var executionBlock: ExecutionBlock?
    private var cancellableTask: Cancellable?

    public init(dispatchQueue: DispatchQueue? = nil, block: @escaping ExecutionBlock) {
        executionBlock = block
        super.init(dispatchQueue: dispatchQueue)
    }

    public convenience init(dispatchQueue: DispatchQueue? = nil, block: @escaping () -> Void) {
        self.init(dispatchQueue: dispatchQueue, block: { operation in
            block()
            operation.finish()
        })
    }

    public convenience init(
        dispatchQueue: DispatchQueue? = nil,
        cancellableTask: @escaping (AsyncBlockOperation) -> Cancellable
    ) {
        self.init(dispatchQueue: dispatchQueue, block: { operation in
            operation.cancellableTask = cancellableTask(operation)
        })
    }

    override public func main() {
        let block = executionBlock
        executionBlock = nil

        assert(block != nil)
        block?(self)
    }

    override public func operationDidCancel() {
        cancellableTask?.cancel()
    }

    override public func operationDidFinish() {
        executionBlock = nil
        cancellableTask = nil
    }
}
