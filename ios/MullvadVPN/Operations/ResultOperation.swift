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

    private let nslock = NSLock()
    private var completionValue: Completion?
    private var _completionQueue: DispatchQueue?
    private var _completionHandler: CompletionHandler?
    private var pendingFinish = false

    var completion: Completion? {
        nslock.lock()
        defer { nslock.unlock() }
        return completionValue
    }

    var completionQueue: DispatchQueue? {
        get {
            nslock.lock()
            defer { nslock.unlock() }

            return _completionQueue
        }
        set {
            nslock.lock()
            _completionQueue = newValue
            nslock.unlock()
        }
    }

    var completionHandler: CompletionHandler? {
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

    override init(dispatchQueue: DispatchQueue?) {
        super.init(dispatchQueue: dispatchQueue)
    }

    init(
        dispatchQueue: DispatchQueue?,
        completionQueue: DispatchQueue?,
        completionHandler: CompletionHandler?
    ) {
        _completionQueue = completionQueue
        _completionHandler = completionHandler

        super.init(dispatchQueue: dispatchQueue)
    }

    @available(*, unavailable)
    override func finish() {
        _finish(error: nil)
    }

    @available(*, unavailable)
    override func finish(error: Error?) {
        _finish(error: error)
    }

    func finish(completion: Completion) {
        nslock.lock()
        if completionValue == nil {
            completionValue = completion
        }
        nslock.unlock()

        _finish(error: completion.error)
    }

    private func _finish(error: Error?) {
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
        let completion = completionValue ?? .cancelled

        // Copy completion queue.
        let completionQueue = _completionQueue
        nslock.unlock()

        let block = {
            // Call completion handler.
            completionHandler?(completion)

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
