//
//  Promise.swift
//  Promise
//
//  Created by pronebird on 03/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

private enum PromiseState<Value> {
    case pending((PromiseResolver<Value>) -> Void, DispatchQueue?)
    case executing
    case resolved(Value, DispatchQueue?)
    case cancelled
}

/// Class describing a block of asynchronous computation that can either resolve or be cancelled.
final class Promise<Value> {
    private var state: PromiseState<Value>
    private var observers: [AnyPromiseObserver<Value>] = []
    private let lock = NSRecursiveLock()

    /// Returns Promise resolved with the given value.
    class func resolved(_ value: Value) -> Self {
        return Self.init(value: value)
    }

    /// Initialize Promise with the execution block.
    init(body: @escaping (PromiseResolver<Value>) -> Void) {
        state = .pending(body, nil)
    }

    /// Initialize resolved Promise with the given value.
    init(value: Value) {
        state = .resolved(value, nil)
    }

    deinit {
        switch state {
        case .resolved, .cancelled:
            break
        case .pending, .executing:
            preconditionFailure("\(Self.self) is deallocated in \(state) state without being resolved or cancelled.")
        }
    }

    /// Observe the result of Promise.
    /// This method starts the promise execution if it hasn't started yet.
    @discardableResult
    func observe(_ receiveCompletion: @escaping (PromiseCompletion<Value>) -> Void) -> Self {
        return lock.withCriticalBlock {
            switch state {
            case .resolved(let value, let queue):
                let completion = PromiseCompletion<Value>.finished(value)
                queue?.async { receiveCompletion(completion) } ?? receiveCompletion(completion)

            case .cancelled:
                receiveCompletion(.cancelled)

            case .pending:
                observers.append(AnyPromiseObserver<Value>(receiveCompletion))
                execute()

            case .executing:
                observers.append(AnyPromiseObserver<Value>(receiveCompletion))
            }
            return self
        }
    }

    /// Cancel Promise.
    /// When Promise is cancelled, all downstream Promises pending execution are also cancelled.
    func cancel() {
        lock.withCriticalBlock {
            switch state {
            case .pending, .executing:
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

    /// Trasform the value by producing a promise.
    func then<NewValue>(_ onResolve: @escaping (Value) -> Promise<NewValue>) -> Promise<NewValue> {
        return Promise<NewValue> { resolver in
            _ = self.observe { completion in
                switch completion {
                case .finished(let value):
                    _ = onResolve(value).observe { completion in
                        resolver.resolve(completion: completion)
                    }
                case .cancelled:
                    resolver.resolve(completion: .cancelled)
                }
            }
        }
    }

    /// Transform the value.
    func then<NewValue>(_ onResolve: @escaping (Value) -> NewValue) -> Promise<NewValue> {
        return Promise<NewValue> { resolver in
            _ = self.observe { completion in
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

    /// Set the queue on which to execute the promise's body block.
    func schedule(on queue: DispatchQueue) -> Self {
        return lock.withCriticalBlock {
            switch state {
            case .pending(let block, _):
                state = .pending(block, queue)
            case .cancelled, .executing, .resolved:
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
        _ = observe { completion in
            returnValue = completion
            condition.signal()
        }

        condition.wait()
        return returnValue
    }

    /// Execute the promise's body if still pending execution.
    private func execute() {
        lock.withCriticalBlock {
            guard case .pending(let block, let queue) = state else { return }

            state = .executing

            let resolver = PromiseResolver(promise: self)

            queue?.async { block(resolver) } ?? block(resolver)
        }
    }

    /// Resolve Promise with the given value.
    ///
    /// Provide the optional `queue` parameter which will be used to dispatch the resolved value to observers added
    /// after the promise was already resolved. When providing a `queue`, the call to `resolve()` must happen on
    /// the same queue.
    fileprivate func resolve(value: Value, queue: DispatchQueue?) {
        lock.withCriticalBlock {
            switch state {
            case .pending, .executing:
                // Oblige caller to resolve the value on the same queue.
                queue.map { dispatchPrecondition(condition: .onQueue($0)) }

                state = .resolved(value, queue)

                observers.forEach { observer in
                    observer.receiveCompletion(.finished(value))
                }
                observers.removeAll()

            case .cancelled, .resolved:
                break
            }

        }
    }

}

final class PromiseCancellationToken {
    private let handler: () -> Void
    fileprivate init(_ handler: @escaping () -> Void) {
        self.handler = handler
    }

    deinit {
        handler()
    }
}

struct PromiseResolver<Value> {
    private let promise: Promise<Value>

    /// Private initializer.
    fileprivate init(promise: Promise<Value>) {
        self.promise = promise
    }

    /// Resolve the promise with `PromiseCompletion`.
    func resolve(completion: PromiseCompletion<Value>) {
        resolve(completion: completion, queue: nil)
    }

    /// Resolve the promise with `PromiseCompletion` and ptiona queue on which to dispatch the value too observers added
    /// after the promise was already resolved.
    func resolve(completion: PromiseCompletion<Value>, queue: DispatchQueue?) {
        switch completion {
        case .finished(let value):
            resolve(value: value, queue: queue)
        case .cancelled:
            promise.cancel()
        }
    }

    /// Resolve Promise with the given value.
    func resolve(value: Value) {
        resolve(value: value, queue: nil)
    }

    /// Resolve the promise with the given value and optional queue on which to dispatch the value to observers added
    /// after the promise was already resolved.
    fileprivate func resolve(value: Value, queue: DispatchQueue?) {
        promise.resolve(value: value, queue: queue)
    }

    /// Set cancellation handler.
    func setCancelHandler(_ cancellation: @escaping () -> Void) {
        _ = promise.observe { completion in
            switch completion {
            case .finished:
                break
            case .cancelled:
                cancellation()
            }
        }
    }
}
