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
public class AsyncBlockOperation: AsyncOperation, @unchecked Sendable {
    private var executor: ((@escaping @Sendable (Error?) -> Void) -> Cancellable?)?
    private var cancellableTask: Cancellable?

    public init(
        dispatchQueue: DispatchQueue? = nil,
        block: @escaping @Sendable (@escaping @Sendable (Error?) -> Void) -> Void
    ) {
        super.init(dispatchQueue: dispatchQueue)
        executor = { finish in
            block(finish)
            return nil
        }
    }

    public init(dispatchQueue: DispatchQueue? = nil, block: @escaping @Sendable () -> Void) {
        super.init(dispatchQueue: dispatchQueue)
        executor = { finish in
            block()
            finish(nil)
            return nil
        }
    }

    public init(
        dispatchQueue: DispatchQueue? = nil,
        cancellableTask: @escaping @Sendable (@escaping @Sendable (Error?) -> Void) -> Cancellable
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
