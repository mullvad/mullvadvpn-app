//
//  ResultBlockOperation.swift
//  Operations
//
//  Created by pronebird on 12/05/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import protocol MullvadTypes.Cancellable

public final class ResultBlockOperation<Success: Sendable>: ResultOperation<Success>, @unchecked Sendable {
    private var executor: ((@escaping @Sendable (Result<Success, Error>) -> Void) -> Cancellable?)?
    private var cancellableTask: Cancellable?

    public init(
        dispatchQueue: DispatchQueue? = nil,
        executionBlock: @escaping @Sendable (_ finish: @escaping (Result<Success, Error>) -> Void) -> Void
    ) {
        super.init(dispatchQueue: dispatchQueue)
        executor = { @Sendable finish in
            executionBlock(finish)
            return nil
        }
    }

    public init(dispatchQueue: DispatchQueue? = nil, executionBlock: @escaping @Sendable () throws -> Success) {
        super.init(dispatchQueue: dispatchQueue)
        executor = { @Sendable finish in
            finish(Result { try executionBlock() })
            return nil
        }
    }

    public init(
        dispatchQueue: DispatchQueue? = nil,
        cancellableTask: @escaping (_ finish: @escaping @Sendable (Result<Success, Error>) -> Void) -> Cancellable
    ) {
        super.init(dispatchQueue: dispatchQueue)
        executor = { cancellableTask($0) }
    }

    override public func main() {
        let executor = executor
        self.executor = nil

        assert(executor != nil)
        cancellableTask = executor?(self.finish)
    }

    override public func operationDidCancel() {
        cancellableTask?.cancel()
    }

    override public func operationDidFinish() {
        executor = nil
        cancellableTask = nil
    }
}
