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

    private let stateLock = NSLock()
    private var completionValue: Completion?
    private let completionQueue: DispatchQueue?
    private var completionHandler: CompletionHandler?
    private var pendingFinish = false

    var completion: Completion? {
        stateLock.lock()
        defer { stateLock.unlock() }
        return completionValue
    }

    init(completionQueue: DispatchQueue?, completionHandler: CompletionHandler?) {
        self.completionQueue = completionQueue
        self.completionHandler = completionHandler

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

    private func _finish() {
        stateLock.lock()
        // Bail if operation is already finishing.
        guard !pendingFinish else {
            stateLock.unlock()
            return
        }

        // Mark that operation is pending finish.
        pendingFinish = true

        // Copy completion handler.
        let completionHandler = self.completionHandler

        // Unset completion handler.
        self.completionHandler = nil

        // Copy completion value.
        let completion = completionValue ?? .cancelled
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
