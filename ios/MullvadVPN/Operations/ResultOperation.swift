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

    override init(dispatchQueue: DispatchQueue?) {
        super.init(dispatchQueue: dispatchQueue)
    }

    init(
        dispatchQueue: DispatchQueue?,
        completionQueue: DispatchQueue?,
        completionHandler: CompletionHandler?
    )
    {
        _completionQueue = completionQueue
        _completionHandler = completionHandler

        super.init(dispatchQueue: dispatchQueue)
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
