//
//  AsyncOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 01/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A base implementation of an asynchronous operation
class AsyncOperation: Operation, OperationProtocol {

    /// A state transaction lock used to perform critical sections of code within `start`, `cancel`
    /// and `finish` calls.
    fileprivate let transactionLock = NSRecursiveLock()

    /// A state lock used for manipulating the operation state flags in a thread safe fashion.
    fileprivate let stateLock = NSRecursiveLock()

    /// The operation observers.
    fileprivate var observers: [AnyOperationObserver<AsyncOperation>] = []

    /// Operation state flags.
    private var _isExecuting = false
    private var _isFinished = false
    private var _isCancelled = false

    final override var isExecuting: Bool {
        return stateLock.withCriticalBlock { _isExecuting }
    }

    final override var isFinished: Bool {
        return stateLock.withCriticalBlock { _isFinished }
    }

    final override var isCancelled: Bool {
        return stateLock.withCriticalBlock { _isCancelled }
    }

    final override var isAsynchronous: Bool {
        return true
    }

    final override func start() {
        transactionLock.withCriticalBlock {
            if self.isCancelled {
                self.finish()
            } else {
                self.setExecuting(true)
                self.main()
            }
        }
    }

    override func main() {
        // Override in subclasses
    }

    /// Cancel operation
    /// Subclasses should override `operationDidCancel` instead
    final override func cancel() {
        transactionLock.withCriticalBlock {
            if self.isCancelled {
                super.cancel()
            } else {
                self.setCancelled(true)

                super.cancel()

                // Only notify the operation about cancellation when it is already running,
                // otherwise the call to `start` should automatically `finish()` the operation.
                if self.isExecuting {
                    self.operationDidCancel()
                }
            }
        }
    }

    /// Override in subclasses to support task cancellation.
    /// Subclasses should call `finish()` to complete the operation
    func operationDidCancel() {
        // no-op
    }

    final func finish() {
        transactionLock.withCriticalBlock {
            guard !self.isFinished else { return }

            self.stateLock.withCriticalBlock {
                self.observers.forEach { $0.operationWillFinish(self) }
            }

            if self.isExecuting {
                self.setExecuting(false)
            }

            self.setFinished(true)

            self.stateLock.withCriticalBlock {
                self.observers.forEach { $0.operationDidFinish(self) }
            }
        }
    }

    private func setExecuting(_ value: Bool) {
        willChangeValue(for: \.isExecuting)
        stateLock.withCriticalBlock { _isExecuting = value }
        didChangeValue(for: \.isExecuting)
    }

    private func setFinished(_ value: Bool) {
        willChangeValue(for: \.isFinished)
        stateLock.withCriticalBlock { _isFinished = value }
        didChangeValue(for: \.isFinished)
    }

    private func setCancelled(_ value: Bool) {
        willChangeValue(for: \.isCancelled)
        stateLock.withCriticalBlock { _isCancelled = value }
        didChangeValue(for: \.isCancelled)
    }

    // MARK: - Observation

    /// Add type-erased operation observer
    fileprivate func addAnyObserver(_ observer: AnyOperationObserver<AsyncOperation>) {
        stateLock.withCriticalBlock {
            self.observers.append(observer)
        }
    }
}

/// This extension exists because Swift has some issues with infering the associated type in `OperationObserver`
extension OperationProtocol where Self: AsyncOperation {
    func addObserver<T: OperationObserver>(_ observer: T) where T.OperationType == Self {
        let transform = TransformOperationObserver<AsyncOperation>(observer)
        let wrapped = AnyOperationObserver(transform)
        addAnyObserver(wrapped)
    }
}


protocol OperationSubclassing {
    /// Use this method in subclasses or extensions where you would like to synchronize
    /// the class members access using the same lock used for guarding from race conditions
    /// when managing operation state.
    func synchronized<T>(_ body: () -> T) -> T
}

extension AsyncOperation: OperationSubclassing {
    func synchronized<T>(_ body: () -> T) -> T {
        return stateLock.withCriticalBlock(body)
    }
}
