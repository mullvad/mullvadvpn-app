//
//  AsyncOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 01/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A base implementation of an asynchronous operation
class AsyncOperation: Operation {
    /// A state lock used for manipulating the operation state flags in a thread safe fashion.
    private let stateLock = NSRecursiveLock()

    /// Operation state flags.
    private var _isExecuting = false
    private var _isFinished = false
    private var _isCancelled = false

    final override var isExecuting: Bool {
        stateLock.lock()
        defer { stateLock.unlock() }

        return _isExecuting
    }

    final override var isFinished: Bool {
        stateLock.lock()
        defer { stateLock.unlock() }

        return _isFinished
    }

    final override var isCancelled: Bool {
        stateLock.lock()
        defer { stateLock.unlock() }

        return _isCancelled
    }

    final override var isAsynchronous: Bool {
        return true
    }

    final override func start() {
        stateLock.lock()
        if _isCancelled {
            stateLock.unlock()
            finish()
        } else {
            setExecuting(true)
            stateLock.unlock()
            main()
        }
    }

    override func main() {
        // Override in subclasses
    }

    override func cancel() {
        stateLock.lock()
        if !_isCancelled {
            willChangeValue(for: \.isCancelled)
            _isCancelled = true
            didChangeValue(for: \.isCancelled)
        }
        stateLock.unlock()

        super.cancel()
    }

    final func finish() {
        stateLock.lock()

        if _isExecuting {
           setExecuting(false)
        }

        if !_isFinished {
            willChangeValue(for: \.isFinished)
            _isFinished = true
            didChangeValue(for: \.isFinished)

            stateLock.unlock()

            operationDidFinish()
        } else {
            stateLock.unlock()
        }
    }

    func operationDidFinish() {
        // Override in subclasses
    }

    private func setExecuting(_ value: Bool) {
        willChangeValue(for: \.isExecuting)
        _isExecuting = value
        didChangeValue(for: \.isExecuting)
    }
}

extension Operation {
    func addDependencies(_ dependencies: [Operation]) {
        for dependency in dependencies {
            addDependency(dependency)
        }
    }
}
