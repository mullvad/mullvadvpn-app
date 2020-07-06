//
//  RetryOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum WaitStrategy {
    case immediate
    case constant(TimeInterval)

    var iterator: AnyIterator<TimeInterval> {
        switch self {
        case .immediate:
            return AnyIterator { .zero }
        case .constant(let constant):
            return AnyIterator { constant }
        }
    }
}

struct RetryStrategy {
    var maxRetries: Int
    var waitStrategy: WaitStrategy
    var waitTimerType: DelayTimerType
}

class RetryOperation<OperationType, Success, Failure: Error>: AsyncOperation, OutputOperation
    where OperationType: OutputOperation, OperationType.Output == Result<Success, Failure>
{
    typealias Output = OperationType.Output

    private let operationQueue = OperationQueue()

    private let producer: () -> OperationType
    private let delayIterator: AnyIterator<TimeInterval>

    private var retryCount: Int = 0
    private let retryStrategy: RetryStrategy

    private var childConfigurator: ((OperationType) -> Void)?

    init(underlyingQueue: DispatchQueue? = nil, strategy: RetryStrategy, producer: @escaping () -> OperationType) {
        operationQueue.underlyingQueue = underlyingQueue
        delayIterator = strategy.waitStrategy.iterator
        retryStrategy = strategy
        self.producer = producer
    }

    override func main() {
        retry()
    }

    override func operationDidCancel() {
        operationQueue.cancelAllOperations()
    }

    private func retry(delay: TimeInterval? = nil) {
        let child = producer()

        child.addDidFinishBlockObserver { [weak self] (operation) in
            guard let self = self else { return }

            // Operation finished without output set?
            guard let result = operation.output else {
                self.finish()
                return
            }

            self.synchronized {
                guard case .failure(let error) = result,
                    let delay = self.delayIterator.next(),
                    self.shouldRetry(error: error) else {
                    self.finish(with: result)
                    return
                }

                self.retryCount += 1
                self.retry(delay: delay)
            }
        }

        synchronized {
            childConfigurator?(child)
        }

        if let delay = delay {
            let delayOperation = DelayOperation(delay: delay, timerType: retryStrategy.waitTimerType)

            child.addDependency(delayOperation)
            operationQueue.addOperation(delayOperation)
        }

        operationQueue.addOperation(child)
    }

    private func setChildConfigurator(_ body: @escaping (OperationType) -> Void) {
        synchronized {
            self.childConfigurator = body
        }
    }

    private func shouldRetry(error: Failure) -> Bool {
        return retryCount < retryStrategy.maxRetries && !self.isCancelled
    }

}

extension RetryOperation: InputOperation where OperationType: InputOperation {
    typealias Input = OperationType.Input

    func operationDidSetInput(_ input: OperationType.Input?) {
        setChildConfigurator { (child) in
            child.input = input
        }
    }
}
