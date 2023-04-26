//
//  ResultBlockOperation.swift
//  Operations
//
//  Created by pronebird on 12/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import protocol MullvadTypes.Cancellable

public final class ResultBlockOperation<Success>: ResultOperation<Success> {
    public typealias ExecutionBlock = (ResultBlockOperation<Success>) -> Void

    private var executionBlock: ExecutionBlock?
    private var cancellableTask: Cancellable?

    public convenience init(dispatchQueue: DispatchQueue? = nil, executionBlock: @escaping ExecutionBlock) {
        self.init(
            dispatchQueue: dispatchQueue,
            executionBlock: executionBlock,
            completionQueue: nil,
            completionHandler: nil
        )
    }

    public convenience init(dispatchQueue: DispatchQueue? = nil, executionBlock: @escaping () throws -> Success) {
        self.init(
            dispatchQueue: dispatchQueue,
            executionBlock: { operation in
                operation.finish(result: Result { try executionBlock() })
            },
            completionQueue: nil,
            completionHandler: nil
        )
    }

    public convenience init(
        dispatchQueue: DispatchQueue? = nil,
        cancellableTask: @escaping (ResultBlockOperation<Success>) -> Cancellable
    ) {
        self.init(
            dispatchQueue: dispatchQueue,
            executionBlock: { operation in
                operation.cancellableTask = cancellableTask(operation)
            },
            completionQueue: nil,
            completionHandler: nil
        )
    }

    public init(
        dispatchQueue: DispatchQueue?,
        executionBlock: @escaping ExecutionBlock,
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
        cancellableTask?.cancel()
    }

    override public func operationDidFinish() {
        executionBlock = nil
        cancellableTask = nil
    }
}
