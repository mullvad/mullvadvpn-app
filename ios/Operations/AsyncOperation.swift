//
//  AsyncOperation.swift
//  Operations
//
//  Created by pronebird on 01/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

@objc private enum State: Int, Comparable, CustomStringConvertible {
    case initialized
    case pending
    case evaluatingConditions
    case ready
    case executing
    case finished

    static func < (lhs: State, rhs: State) -> Bool {
        return lhs.rawValue < rhs.rawValue
    }

    var description: String {
        switch self {
        case .initialized:
            return "initialized"
        case .pending:
            return "pending"
        case .evaluatingConditions:
            return "evaluatingConditions"
        case .ready:
            return "ready"
        case .executing:
            return "executing"
        case .finished:
            return "finished"
        }
    }
}

/// A base implementation of an asynchronous operation
open class AsyncOperation: Operation {
    /// Mutex lock used for guarding critical sections of operation lifecycle.
    private let operationLock = NSRecursiveLock()

    /// Mutex lock used to guard `state` and `isCancelled` properties.
    ///
    /// This lock must not encompass KVO hooks such as `willChangeValue` and `didChangeValue` to
    /// prevent deadlocks, since KVO observers may synchronously query the operation state on a
    /// different thread.
    ///
    /// `operationLock` should be used along with `stateLock` to ensure internal state consistency
    /// when multiple access to `state` or `isCancelled` is necessary, such as when testing
    /// the value before modifying it.
    private let stateLock = NSRecursiveLock()

    /// Backing variable for `state`.
    /// Access must be guarded with `stateLock`.
    private var _state: State = .initialized

    /// Backing variable for `_isCancelled`.
    /// Access must be guarded with `stateLock`.
    private var __isCancelled = false

    /// Backing variable for `error`.
    /// Access must be guarded with `stateLock`.
    private var __error: Error?

    /// Operation state.
    @objc private var state: State {
        get {
            stateLock.lock()
            defer { stateLock.unlock() }

            return _state
        }
        set(newState) {
            willChangeValue(for: \.state)
            stateLock.lock()
            assert(_state < newState)
            _state = newState
            stateLock.unlock()
            didChangeValue(for: \.state)
        }
    }

    private var _isCancelled: Bool {
        get {
            stateLock.lock()
            defer { stateLock.unlock() }

            return __isCancelled
        }
        set {
            willChangeValue(for: \.isCancelled)
            stateLock.lock()
            __isCancelled = newValue
            stateLock.unlock()
            didChangeValue(for: \.isCancelled)
        }
    }

    private var _error: Error? {
        get {
            stateLock.lock()
            defer { stateLock.unlock() }
            return __error
        }
        set {
            stateLock.lock()
            defer { stateLock.unlock() }
            __error = newValue
        }
    }

    public var error: Error? {
        return _error
    }

    override public final var isReady: Bool {
        stateLock.lock()
        defer { stateLock.unlock() }

        // super.isReady should turn true when all dependencies are satisfied.
        guard super.isReady else {
            return false
        }

        // Mark operation ready when cancelled, so that operation queue could flush it faster.
        guard !__isCancelled else {
            return true
        }

        switch _state {
        case .initialized, .pending, .evaluatingConditions:
            return false

        case .ready, .executing, .finished:
            return true
        }
    }

    override public final var isExecuting: Bool {
        return state == .executing
    }

    override public final var isFinished: Bool {
        return state == .finished
    }

    override public final var isCancelled: Bool {
        return _isCancelled
    }

    override public final var isAsynchronous: Bool {
        return true
    }

    // MARK: - Observers

    private var _observers: [OperationObserver] = []

    public final var observers: [OperationObserver] {
        operationLock.lock()
        defer { operationLock.unlock() }

        return _observers
    }

    public final func addObserver(_ observer: OperationObserver) {
        operationLock.lock()
        assert(state < .executing)
        _observers.append(observer)
        operationLock.unlock()
        observer.didAttach(to: self)
    }

    // MARK: - Conditions

    private var _conditions: [OperationCondition] = []

    public final var conditions: [OperationCondition] {
        operationLock.lock()
        defer { operationLock.unlock() }

        return _conditions
    }

    public func addCondition(_ condition: OperationCondition) {
        operationLock.lock()
        assert(state < .evaluatingConditions)
        _conditions.append(condition)
        operationLock.unlock()
    }

    private func evaluateConditions() {
        guard !_conditions.isEmpty else {
            state = .ready
            return
        }

        state = .evaluatingConditions

        var results = [Bool](repeating: false, count: _conditions.count)
        let group = DispatchGroup()

        for (index, condition) in _conditions.enumerated() {
            group.enter()
            condition.evaluate(for: self) { [weak self] isSatisfied in
                self?.dispatchQueue.async {
                    results[index] = isSatisfied
                    group.leave()
                }
            }
        }

        group.notify(queue: dispatchQueue) { [weak self] in
            self?.didEvaluateConditions(results)
        }
    }

    private func didEvaluateConditions(_ results: [Bool]) {
        operationLock.lock()
        defer { operationLock.unlock() }

        guard state < .ready else { return }

        let conditionsSatisfied = results.allSatisfy { $0 }
        if !conditionsSatisfied {
            cancel()
        }

        state = .ready
    }

    // MARK: -

    public let dispatchQueue: DispatchQueue

    public init(dispatchQueue: DispatchQueue? = nil) {
        self.dispatchQueue = dispatchQueue ?? DispatchQueue(label: "AsyncOperation.dispatchQueue")
        super.init()

        addObserver(
            self,
            forKeyPath: #keyPath(isReady),
            options: [],
            context: &Self.observerContext
        )
    }

    deinit {
        removeObserver(self, forKeyPath: #keyPath(isReady), context: &Self.observerContext)
    }

    // MARK: - KVO

    private static var observerContext = 0

    override public func observeValue(
        forKeyPath keyPath: String?,
        of object: Any?,
        change: [NSKeyValueChangeKey: Any]?,
        context: UnsafeMutableRawPointer?
    ) {
        if context == &Self.observerContext {
            checkReadiness()
            return
        }

        super.observeValue(
            forKeyPath: keyPath,
            of: object,
            change: change,
            context: context
        )
    }

    @objc class func keyPathsForValuesAffectingIsReady() -> Set<String> {
        return [#keyPath(state)]
    }

    @objc class func keyPathsForValuesAffectingIsExecuting() -> Set<String> {
        return [#keyPath(state)]
    }

    @objc class func keyPathsForValuesAffectingIsFinished() -> Set<String> {
        return [#keyPath(state)]
    }

    // MARK: - Lifecycle

    override public final func start() {
        let currentQueue = OperationQueue.current
        let underlyingQueue = currentQueue?.underlyingQueue

        if underlyingQueue == dispatchQueue {
            _start()
        } else {
            dispatchQueue.async {
                self._start()
            }
        }
    }

    private func _start() {
        operationLock.lock()
        if _isCancelled {
            operationLock.unlock()
            finish()
        } else {
            state = .executing

            for observer in _observers {
                observer.operationDidStart(self)
            }
            operationLock.unlock()

            main()
        }
    }

    override open func main() {
        // Override in subclasses
    }

    override public final func cancel() {
        var notifyDidCancel = false

        operationLock.lock()
        if !_isCancelled {
            _isCancelled = true
            notifyDidCancel = true
        }
        operationLock.unlock()

        super.cancel()

        if notifyDidCancel {
            dispatchQueue.async {
                self.operationDidCancel()

                for observer in self.observers {
                    observer.operationDidCancel(self)
                }
            }
        }
    }

    public func finish() {
        finish(error: nil)
    }

    public func finish(error: Error?) {
        guard tryFinish(error: error) else { return }

        dispatchQueue.async {
            self.operationDidFinish()

            let anError = self.error
            for observer in self.observers {
                observer.operationDidFinish(self, error: anError)
            }
        }
    }

    // MARK: - Private

    internal func didEnqueue() {
        operationLock.lock()
        defer { operationLock.unlock() }

        guard state == .initialized else {
            return
        }

        state = .pending
    }

    private func checkReadiness() {
        operationLock.lock()
        defer { operationLock.unlock() }

        if state == .pending, !_isCancelled, super.isReady {
            evaluateConditions()
        }
    }

    private func tryFinish(error: Error?) -> Bool {
        operationLock.lock()
        defer { operationLock.unlock() }

        guard state < .finished else { return false }

        _error = error
        state = .finished

        return true
    }

    // MARK: - Subclass overrides

    open func operationDidCancel() {
        // Override in subclasses.
    }

    open func operationDidFinish() {
        // Override in subclasses.
    }
}

extension Operation {
    public func addDependencies(_ dependencies: [Operation]) {
        for dependency in dependencies {
            addDependency(dependency)
        }
    }
}

public protocol OperationBlockObserverSupport {}
extension AsyncOperation: OperationBlockObserverSupport {}

extension OperationBlockObserverSupport where Self: AsyncOperation {
    public func addBlockObserver(_ observer: OperationBlockObserver<Self>) {
        addObserver(observer)
    }
}
