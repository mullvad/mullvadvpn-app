//
//  ResultOperation.swift
//  Operations
//
//  Created by pronebird on 23/03/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Base class for operations producing result.
open class ResultOperation<Success>: AsyncOperation, OutputOperation {
    public typealias CompletionHandler = (Result<Success, Error>) -> Void

    private let nslock = NSLock()
    private var _output: Success?
    private var _completionQueue: DispatchQueue?
    private var _completionHandler: CompletionHandler?
    private var pendingFinish = false

    public var result: Result<Success, Error>? {
        nslock.lock()
        defer { nslock.unlock() }

        return _output.map { .success($0) } ?? error.map { .failure($0) }
    }

    public var output: Success? {
        nslock.lock()
        defer { nslock.unlock() }

        return _output
    }

    public var completionQueue: DispatchQueue? {
        get {
            nslock.lock()
            defer { nslock.unlock() }

            return _completionQueue
        }
        set {
            nslock.lock()
            defer { nslock.unlock() }

            _completionQueue = newValue
        }
    }

    public var completionHandler: CompletionHandler? {
        get {
            nslock.lock()
            defer { nslock.unlock() }

            return _completionHandler
        }
        set {
            nslock.lock()
            defer { nslock.unlock() }
            if !pendingFinish {
                _completionHandler = newValue
            }
        }
    }

    override public init(dispatchQueue: DispatchQueue?) {
        super.init(dispatchQueue: dispatchQueue)
    }

    public init(
        dispatchQueue: DispatchQueue?,
        completionQueue: DispatchQueue?,
        completionHandler: CompletionHandler?
    ) {
        _completionQueue = completionQueue
        _completionHandler = completionHandler

        super.init(dispatchQueue: dispatchQueue)
    }

    @available(*, unavailable)
    override public func finish() {
        _finish(result: .failure(OperationError.cancelled))
    }

    @available(*, unavailable)
    override public func finish(error: Error?) {
        _finish(result: .failure(error ?? OperationError.cancelled))
    }

    open func finish(result: Result<Success, Error>) {
        _finish(result: result)
    }

    private func _finish(result: Result<Success, Error>) {
        nslock.lock()
        // Bail if operation is already finishing.
        guard !pendingFinish else {
            nslock.unlock()
            return
        }

        // Mark that operation is pending finish.
        pendingFinish = true

        // Copy completion handler.
        let completionHandler = _completionHandler

        // Unset completion handler.
        _completionHandler = nil

        // Copy completion value.
        if case let .success(output) = result {
            _output = output
        }

        // Copy completion queue.
        let completionQueue = _completionQueue
        nslock.unlock()

        let block = {
            // Call completion handler.
            completionHandler?(result)

            var error: Error?
            if case let .failure(failure) = result {
                error = failure
            }

            // Finish operation.
            super.finish(error: error)
        }

        if let completionQueue = completionQueue {
            completionQueue.async(execute: block)
        } else {
            block()
        }
    }
}
