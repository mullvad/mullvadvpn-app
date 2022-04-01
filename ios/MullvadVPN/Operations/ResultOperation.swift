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
        // Propagate cancellation if finish() is called directly from start().
        if isCancelled {
            finish(completion: .cancelled)
        } else {
            preconditionFailure("Use finish(completion:) to finish operation.")
        }
    }

    func finish(completion: Completion) {
        stateLock.lock()

        // Bail if operation is already finishing.
        guard completionValue == nil else {
            stateLock.unlock()
            return
        }

        // Store completion value.
        completionValue = completion

        // Copy completion handler.
        let completionHandler: CompletionHandler? = self.completionHandler

        // Unset completion handler.
        self.completionHandler = nil

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
