//
//  Promise.swift
//  Promise
//
//  Created by pronebird on 03/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Enum describing the state of the Promise lifecycle.
private enum PromiseState<Value> {
    case pending((PromiseResolver<Value>) -> Void)
    case executing
    case resolved(Value)
    case cancelling
    case cancelled
}

/// Class describing a block of asynchronous computation that can either resolve or be cancelled.
final class Promise<Value>: Cancellable {
    private var state: PromiseState<Value>
    private var observers: [AnyPromiseObserver<Value>] = []
    private let lock = NSRecursiveLock()

    /// Execution queue used for running the Promise body.
    private var executionQueue: DispatchQueue?

    /// Completion queue used for delivering results to observers.
    private var completionQueue: DispatchQueue?

    /// Parent promise.
    private var parent: Cancellable?

    /// Cancellation handler.
    private var cancelHandler: (() -> Void)?

    /// Whether to propagate cancellation to the parent promise.
    private var shouldPropagateCancellation = true

    /// Returns Promise resolved with the given value.
    class func resolved(_ value: Value) -> Self {
        return Self.init(value: value)
    }

    /// Returns Promise with lazily resolved value.
    class func deferred(_ producer: @escaping () -> Value) -> Self {
        return Self.init { resolver in
            resolver.resolve(value: producer())
        }
    }

    /// Initialize Promise with the execution block.
    init(body: @escaping (PromiseResolver<Value>) -> Void) {
        state = .pending(body)
    }

    /// Initialize Promise with the execution block and parent.
    init(parent aParent: Cancellable?, body: @escaping (PromiseResolver<Value>) -> Void) {
        state = .pending(body)
        parent = aParent
    }

    /// Initialize resolved Promise with the given value.
    init(value: Value) {
        state = .resolved(value)
    }

    deinit {
        switch state {
        case .resolved, .cancelled, .pending:
            break
        case .executing, .cancelling:
            preconditionFailure("\(Self.self) is deallocated in \(state) state without being resolved or cancelled.")
        }
    }

    /// Observe the result of Promise.
    /// This method starts the promise execution if it hasn't started yet.
    func observe(_ receiveCompletion: @escaping (PromiseCompletion<Value>) -> Void) {
        return lock.withCriticalBlock {
            switch state {
            case .resolved(let value):
                let completion = PromiseCompletion<Value>.finished(value)
                completionQueue?.async { receiveCompletion(completion) } ?? receiveCompletion(completion)

            case .cancelled:
                let completion = PromiseCompletion<Value>.cancelled
                completionQueue?.async { receiveCompletion(completion) } ?? receiveCompletion(completion)

            case .pending:
                observers.append(AnyPromiseObserver<Value>(receiveCompletion))
                execute()

            case .executing, .cancelling:
                observers.append(AnyPromiseObserver<Value>(receiveCompletion))
            }
        }
    }

    /// Cancel Promise.
    func cancel() {
        lock.withCriticalBlock {
            switch state {
            case .pending:
                state = .cancelled

            case .executing:
                state = .cancelling

                if shouldPropagateCancellation {
                    parent?.cancel()
                }

                triggerCancelHandler()

            case .cancelling, .cancelled, .resolved:
                break
            }
        }
    }

    /// Trasform the value by producing a promise.
    func then<NewValue>(_ onResolve: @escaping (Value) -> Promise<NewValue>) -> Promise<NewValue> {
        return Promise<NewValue>(parent: self) { resolver in
            self.observe { completion in
                switch completion {
                case .finished(let value):
                    let child = onResolve(value)

                    resolver.setCancelHandler {
                        child.cancel()
                    }

                    child.observe { completion in
                        resolver.resolve(completion: completion)
                    }

                case .cancelled:
                    resolver.resolve(completion: .cancelled)
                }
            }
        }
    }

    /// Transform the value producing new value.
    func then<NewValue>(_ onResolve: @escaping (Value) -> NewValue) -> Promise<NewValue> {
        return Promise<NewValue>(parent: self) { resolver in
            self.observe { completion in
                resolver.resolve(completion: completion.map(onResolve))
            }
        }
    }

    /// Assign the cancellation token into the given variable.
    /// Releasing the cancellation token cancels the given Promise and all downstream Promises.
    func storeCancellationToken(in token: inout PromiseCancellationToken?) -> Self {
        token = PromiseCancellationToken { [weak self] in
            self?.cancel()
        }
        return self
    }

    /// Switch the cancellation propagation behaviour
    func setShouldPropagateCancellation(_ propagateCancellation: Bool) -> Self {
        return lock.withCriticalBlock {
            shouldPropagateCancellation = propagateCancellation
            return self
        }
    }

    /// Set the queue on which to execute the promise's body block.
    func schedule(on queue: DispatchQueue) -> Self {
        return lock.withCriticalBlock {
            switch state {
            case .pending:
                executionQueue = queue
            case .cancelling, .cancelled, .executing, .resolved:
                break
            }
            return self
        }
    }

    /// Block the given queue until the promise finished executing.
    func block(on dispatchQueue: DispatchQueue) -> Promise<Value> {
        return Promise { resolver in
            dispatchQueue.async {
                let completion = self.await()

                resolver.resolve(completion: completion)
            }
        }
    }

    /// Block current queue until the promise finished executing.
    func await() -> PromiseCompletion<Value> {
        let condition = NSCondition()
        condition.lock()
        defer { condition.unlock() }

        var returnValue: PromiseCompletion<Value>!
        observe { completion in
            returnValue = completion
            condition.signal()
        }

        condition.wait()
        return returnValue
    }

    // MARK: - Private

    /// Execute the promise's body if still pending execution.
    private func execute() {
        lock.withCriticalBlock {
            guard case .pending(let block) = state else { return }

            state = .executing

            let resolver = PromiseResolver(promise: self)

            executionQueue?.async { block(resolver) } ?? block(resolver)
        }
    }

    /// Resolve Promise with `PromiseCompletion`.
    fileprivate func resolve(completion: PromiseCompletion<Value>, queue: DispatchQueue?) {
        lock.withCriticalBlock {
            switch completion {
            case .finished(let value):
                resolve(value: value, queue: queue)
            case .cancelled:
                resolveCancelled(queue: queue)
            }
        }
    }

    /// Resolve Promise with the given value.
    ///
    /// Provide the optional `queue` parameter which will be used to dispatch the resolved value to observers added
    /// after the promise was already resolved. When providing a `queue`, the call to `resolve()` must happen on
    /// the same queue.
    private func resolve(value: Value, queue: DispatchQueue?) {
        lock.withCriticalBlock {
            switch state {
            case .pending, .executing:
                // Oblige caller to resolve the value on the same queue.
                queue.map { dispatchPrecondition(condition: .onQueue($0)) }

                completionQueue = queue
                state = .resolved(value)

                observers.forEach { observer in
                    observer.receiveCompletion(.finished(value))
                }
                observers.removeAll()

            case .cancelling:
                // Oblige caller to resolve the value on the same queue.
                queue.map { dispatchPrecondition(condition: .onQueue($0)) }

                completionQueue = queue
                state = .cancelled

                observers.forEach { observer in
                    observer.receiveCompletion(.cancelled)
                }
                observers.removeAll()

            case .cancelled, .resolved:
                break
            }
        }
    }

    private func resolveCancelled(queue: DispatchQueue?) {
        lock.withCriticalBlock {
            switch state {
            case .pending, .executing, .cancelling:
                // Oblige caller to resolve the value on the same queue.
                queue.map { dispatchPrecondition(condition: .onQueue($0)) }

                completionQueue = queue
                state = .cancelled

                observers.forEach { observer in
                    observer.receiveCompletion(.cancelled)
                }
                observers.removeAll()

            case .cancelled, .resolved:
                break
            }
        }
    }

    /// Set cancellation handler.
    fileprivate func setCancelHandler(_ handler: @escaping () -> Void) {
        lock.withCriticalBlock {
            cancelHandler = handler
        }
    }

    /// Trigger cancellation handler, then reset it.
    private func triggerCancelHandler() {
        lock.withCriticalBlock {
            let cancelHandlerCopy = cancelHandler
            cancelHandler = nil
            cancelHandlerCopy?()
        }
    }

}

final class PromiseCancellationToken {
    private var handler: (() -> Void)?
    private let lock = NSLock()

    fileprivate init(_ aHandler: @escaping () -> Void) {
        handler = aHandler
    }

    func cancel() {
        lock.withCriticalBlock {
            handler?()
            handler = nil
        }
    }

    deinit {
        cancel()
    }
}

struct PromiseResolver<Value> {
    /// Target promise.
    private let promise: Promise<Value>

    /// Private initializer.
    fileprivate init(promise aPromise: Promise<Value>) {
        promise = aPromise
    }

    /// Resolve the promise with `PromiseCompletion` and optional queue on which to dispatch the value to observers
    /// added after the promise was already resolved.
    func resolve(completion: PromiseCompletion<Value>, queue: DispatchQueue? = nil) {
        promise.resolve(completion: completion, queue: queue)
    }

    /// Resolve the promise with the given value and optional queue on which to dispatch the value to observers added
    /// after the promise was already resolved.
    func resolve(value: Value, queue: DispatchQueue? = nil) {
        promise.resolve(completion: .finished(value), queue: queue)
    }

    /// Set cancellation handler.
    func setCancelHandler(_ handler: @escaping () -> Void) {
        promise.setCancelHandler(handler)
    }
}
