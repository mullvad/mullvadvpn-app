//
//  ResultOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 23/03/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Base class for operations producing result.
class ResultOperation<Success, Failure: Error>: AsyncOperation {
    typealias Completion = OperationCompletion<Success, Failure>
    typealias CompletionHandler = (Completion) -> Void

    fileprivate let stateLock = NSLock()
    private var completionValue: Completion?
    private var _completionQueue: DispatchQueue?
    private var _completionHandler: CompletionHandler?
    private var pendingFinish = false

    var completion: Completion? {
        stateLock.lock()
        defer { stateLock.unlock() }
        return completionValue
    }

    var completionQueue: DispatchQueue? {
        get {
            stateLock.lock()
            defer { stateLock.unlock() }

            return _completionQueue
        }
        set {
            stateLock.lock()
            _completionQueue = newValue
            stateLock.unlock()
        }
    }

    var completionHandler: CompletionHandler? {
        get {
            stateLock.lock()
            defer { stateLock.unlock() }

            return _completionHandler
        }
        set {
            stateLock.lock()
            defer { stateLock.unlock() }
            if !pendingFinish {
                _completionHandler = newValue
            }
        }
    }

    init(completionQueue: DispatchQueue?, completionHandler: CompletionHandler?) {
        _completionQueue = completionQueue
        _completionHandler = completionHandler

        super.init()
    }

    @available(*, unavailable)
    override func finish() {
        _finish()
    }

    func finish(completion: Completion) {
        stateLock.lock()
        if completionValue == nil {
            completionValue = completion
        }
        stateLock.unlock()

        _finish()
    }

    fileprivate func _finish() {
        stateLock.lock()
        // Bail if operation is already finishing.
        guard !pendingFinish else {
            stateLock.unlock()
            return
        }

        // Mark that operation is pending finish.
        pendingFinish = true

        // Copy completion handler.
        let completionHandler = _completionHandler

        // Unset completion handler.
        _completionHandler = nil

        // Copy completion value.
        let completion = completionValue ?? .cancelled

        // Copy completion queue.
        let completionQueue = _completionQueue
        stateLock.unlock()

        let block = {
            // Call completion handler.
            completionHandler?(completion)

            // Finish operation.
            super.finish()
        }

        if let completionQueue = completionQueue {
            completionQueue.async(execute: block)
        } else {
            block()
        }
    }
}

class ResultBlockOperation<Success, Failure: Error>: ResultOperation<Success, Failure> {
    typealias ExecutionBlock = (ResultBlockOperation<Success, Failure>) -> Void

    private var executionBlock: ExecutionBlock?
    private var cancellationBlocks: [() -> Void] = []

    convenience init(executionBlock: @escaping ExecutionBlock) {
        self.init(
            executionBlock: executionBlock,
            completionQueue: nil,
            completionHandler: nil
        )
    }

    init(
        executionBlock: @escaping ExecutionBlock,
        completionQueue: DispatchQueue?,
        completionHandler: CompletionHandler?
    )
    {
        self.executionBlock = executionBlock
        super.init(completionQueue: completionQueue, completionHandler: completionHandler)
    }

    override func main() {
        stateLock.lock()
        let block = executionBlock
        executionBlock = nil
        stateLock.unlock()

        block?(self)
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

    override func _finish() {
        stateLock.lock()
        cancellationBlocks.removeAll()
        executionBlock = nil
        stateLock.unlock()

        super._finish()
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

