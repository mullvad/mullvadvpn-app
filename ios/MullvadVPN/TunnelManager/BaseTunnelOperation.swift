//
//  BaseTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 25/01/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

class BaseTunnelOperation<Success, Failure: Error>: AsyncOperation {
    private var observers = [TunnelOperationObserver<Success, Failure>]()
    private let observerLock = NSLock()

    let queue: DispatchQueue

    init(queue: DispatchQueue) {
        self.queue = queue
    }

    private func addObserver(_ observer: TunnelOperationObserver<Success, Failure>) {
        assert(!isExecuting && !isFinished)

        observerLock.withCriticalBlock {
            observers.append(observer)
        }
    }

    func addObserver(queue: DispatchQueue?, finishHandler: @escaping (Operation, OperationCompletion<Success, Failure>) -> Void) {
        addObserver(TunnelOperationObserver(queue: queue, finishHandler: finishHandler))
    }

    private func notifyObservers(completion: OperationCompletion<Success, Failure>) {
        observerLock.withCriticalBlock {
            observers.forEach { observer in
                observer.operationDidFinish(self, completion: completion)
            }
            observers.removeAll()
        }
    }

    func completeOperation(completion: OperationCompletion<Success, Failure>) {
        notifyObservers(completion: completion)

        finish()
    }
}
