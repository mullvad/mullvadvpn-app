//
//  AsyncOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 01/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

@objc enum State: Int, Comparable, CustomStringConvertible {
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
class AsyncOperation: Operation {
    /// A state lock used for manipulating the operation state in a thread safe fashion.
    private let stateLock = NSRecursiveLock()

    /// Operation state.
    @objc private var state: State = .initialized
    private var _isCancelled = false

    final override var isReady: Bool {
        stateLock.lock()
        defer { stateLock.unlock() }

        // super.isReady should turn true when all dependencies are satisfied.
        guard super.isReady else {
            return false
        }

        // Mark operation ready when cancelled, so that operation queue could flush it faster.
        guard !_isCancelled else {
            return true
        }

        switch state {
        case .initialized, .pending, .evaluatingConditions:
            return false

        case .ready, .executing, .finished:
            return true
        }
    }

    final override var isExecuting: Bool {
        stateLock.lock()
        defer { stateLock.unlock() }

        return state == .executing
    }

    final override var isFinished: Bool {
        stateLock.lock()
        defer { stateLock.unlock() }

        return state == .finished
    }

    final override var isCancelled: Bool {
        stateLock.lock()
        defer { stateLock.unlock() }

        return _isCancelled
    }

    final override var isAsynchronous: Bool {
        return true
    }

    // MARK: - Observers

    private var _observers: [OperationObserver] = []

    final var observers: [OperationObserver] {
        stateLock.lock()
        defer { stateLock.unlock() }

        return _observers
    }

    final func addObserver(_ observer: OperationObserver) {
        stateLock.lock()
        assert(state < .executing)
        _observers.append(observer)
        stateLock.unlock()
        observer.didAttach(to: self)
    }

    // MARK: - Conditions

    private var _conditions: [OperationCondition] = []

    final var conditions: [OperationCondition] {
        stateLock.lock()
        defer { stateLock.unlock() }

        return _conditions
    }

    func addCondition(_ condition: OperationCondition) {
        stateLock.lock()
        assert(state < .evaluatingConditions)
        _conditions.append(condition)
        stateLock.unlock()
    }

    private func evaluateConditions() {
        guard !_conditions.isEmpty else {
            setState(.ready)
            return
        }

        setState(.evaluatingConditions)

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
        stateLock.lock()
        defer { stateLock.unlock() }

        guard state < .ready else { return }

        let conditionsSatisfied = results.allSatisfy { $0 }
        if !conditionsSatisfied {
            cancel()
        }

        setState(.ready)
    }

    // MARK: -

    let dispatchQueue: DispatchQueue

    init(dispatchQueue: DispatchQueue? = nil) {
        self.dispatchQueue = dispatchQueue ?? DispatchQueue(label: "AsyncOperation.dispatchQueue")
        super.init()
    }

    // MARK: - KVO

    @objc class func keyPathsForValuesAffectingIsReady() -> Set<String> {
        return ["state"]
    }

    @objc class func keyPathsForValuesAffectingIsExecuting() -> Set<String> {
        return ["state"]
    }

    @objc class func keyPathsForValuesAffectingIsFinished() -> Set<String> {
        return ["state"]
    }

    // MARK: - Lifecycle

    final override func start() {
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
        stateLock.lock()
        if _isCancelled {
            stateLock.unlock()
            finish()
        } else {
            setState(.executing)

            for observer in _observers {
                observer.operationDidStart(self)
            }
            stateLock.unlock()

            main()
        }
    }

    override func main() {
        // Override in subclasses
    }

    final override func cancel() {
        var notifyDidCancel = false

        stateLock.lock()
        if !_isCancelled {
            willChangeValue(for: \.isCancelled)
            _isCancelled = true
            didChangeValue(for: \.isCancelled)

            notifyDidCancel = true
        }
        stateLock.unlock()

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

    func finish() {
        var notifyDidFinish = false

        stateLock.lock()
        if state < .finished {
            setState(.finished)
            notifyDidFinish = true
        }
        stateLock.unlock()

        if notifyDidFinish {
            dispatchQueue.async {
                self.operationDidFinish()

                for observer in self.observers {
                    observer.operationDidFinish(self)
                }
            }
        }
    }

    // MARK: - Private

    private func setState(_ newState: State) {
        willChangeValue(for: \.state)
        assert(state < newState)
        state = newState
        didChangeValue(for: \.state)
    }

    private func dependenciesDidFinish() {
        stateLock.lock()
        defer { stateLock.unlock() }

        guard state == .pending && !_isCancelled else { return }

        precondition(super.isReady, "Expect super.isReady to be true.")

        evaluateConditions()
    }

    func didEnqueue() {
        stateLock.lock()
        guard state == .initialized else {
            stateLock.unlock()
            return
        }
        setState(.pending)
        stateLock.unlock()

        let group = DispatchGroup()
        var observers = [NSKeyValueObservation]()

        for dependency in dependencies {
            group.enter()

            let observer = dependency.observe(\.isFinished, options: [.initial]) { dependency, _ in
                if dependency.isFinished {
                    group.leave()
                }
            }

            observers.append(observer)
        }

        group.notify(queue: dispatchQueue) {
            for observer in observers {
                observer.invalidate()
            }

            self.dependenciesDidFinish()
        }
    }

    // MARK: - Subclass overrides

    func operationDidCancel() {
        // Override in subclasses.
    }

    func operationDidFinish() {
        // Override in subclasses.
    }
}

extension Operation {
    func addDependencies(_ dependencies: [Operation]) {
        for dependency in dependencies {
            addDependency(dependency)
        }
    }
}

extension Operation {
    var operationName: String {
        return name ?? "\(self)"
    }
}


protocol OperationBlockObserverSupport {}
extension AsyncOperation: OperationBlockObserverSupport {}

extension OperationBlockObserverSupport where Self: AsyncOperation {
    func addBlockObserver(_ observer: OperationBlockObserver<Self>) {
        addObserver(observer)
    }
}
