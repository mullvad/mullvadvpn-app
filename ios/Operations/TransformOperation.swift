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

    private var executor: ((Input, @escaping (Result<Output, Error>) -> Void) -> Cancellable?)?
    private var cancellableTask: Cancellable?

    public init(
        dispatchQueue: DispatchQueue? = nil,
        input: Input? = nil,
        block: @escaping (_ input: Input, _ finish: @escaping (Result<Output, Error>) -> Void) -> Void
    ) {
        super.init(dispatchQueue: dispatchQueue)
        __input = input
        executor = { input, finish in
            block(input, finish)
            return nil
        }
    }

    public init(
        dispatchQueue: DispatchQueue? = nil,
        input: Input? = nil,
        throwingBlock: @escaping (_ input: Input) throws -> Output
    ) {
        super.init(dispatchQueue: dispatchQueue)
        __input = input
        executor = { input, finish in
            finish(Result { try throwingBlock(input) })
            return nil
        }
    }

    public init(
        dispatchQueue: DispatchQueue? = nil,
        input: Input? = nil,
        cancellableTask: @escaping (_ input: Input, _ finish: @escaping (Result<Output, Error>) -> Void) -> Cancellable
    ) {
        super.init(dispatchQueue: dispatchQueue)
        __input = input
        executor = cancellableTask
    }

    override public func main() {
        if let inputBlock {
            _input = inputBlock()
        }

        guard let inputValue = _input else {
            finish(result: .failure(OperationError.unsatisfiedRequirement))
            return
        }

        let executor = executor
        self.executor = nil

        assert(executor != nil)

        cancellableTask = executor?(inputValue, self.finish)
    }

    override public func operationDidCancel() {
        cancellableTask?.cancel()
    }

    override public func operationDidFinish() {
        executor = nil
        cancellableTask = nil
    }

    // MARK: - Input injection

    public func setInputBlock(_ block: @escaping () -> Input?) {
        dispatchQueue.async {
            self.inputBlock = block
        }
    }
}
