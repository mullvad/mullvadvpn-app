//
//  TransformOperation.swift
//  Operations
//
//  Created by pronebird on 31/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import protocol MullvadTypes.Cancellable

public final class TransformOperation<Input, Output>: ResultOperation<Output>, InputOperation {
    public typealias ExecutionBlock = (Input, TransformOperation<Input, Output>) -> Void
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
    private var cancellableTask: Cancellable?

    public init(dispatchQueue: DispatchQueue? = nil, input: Input? = nil, block: ExecutionBlock? = nil) {
        __input = input
        executionBlock = block

        super.init(dispatchQueue: dispatchQueue)
    }

    public convenience init(
        dispatchQueue: DispatchQueue? = nil,
        input: Input? = nil,
        throwingBlock: @escaping (Input) throws -> Output
    ) {
        self.init(dispatchQueue: dispatchQueue, input: input, block: { input, operation in
            operation.finish(result: Result { try throwingBlock(input) })
        })
    }

    public convenience init(
        dispatchQueue: DispatchQueue? = nil,
        input: Input? = nil,
        cancellableTask: @escaping (Input, TransformOperation<Input, Output>) -> Cancellable
    ) {
        self.init(dispatchQueue: dispatchQueue, input: input, block: { input, operation in
            operation.cancellableTask = cancellableTask(input, operation)
        })
    }

    override public func main() {
        if let inputBlock = inputBlock {
            _input = inputBlock()
        }

        guard let inputValue = _input, let executionBlock = executionBlock else {
            finish(result: .failure(OperationError.unsatisfiedRequirement))
            return
        }

        executionBlock(inputValue, self)
    }

    override public func operationDidCancel() {
        cancellableTask?.cancel()
    }

    override public func operationDidFinish() {
        executionBlock = nil
        cancellableTask = nil
    }

    // MARK: - Input injection

    public func setInputBlock(_ block: @escaping () -> Input?) {
        dispatchQueue.async {
            self.inputBlock = block
        }
    }
}
