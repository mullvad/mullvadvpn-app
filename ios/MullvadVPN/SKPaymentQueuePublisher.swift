//
//  SKPaymentQueuePublisher.swift
//  MullvadVPN
//
//  Created by pronebird on 17/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import StoreKit

/// A publisher that indefinitely emits the incoming transactions on the given `SKPaymentQueue`,
/// and never completes.
struct SKPaymentQueuePublisher: Publisher {
    typealias Output = SKPaymentTransaction
    typealias Failure = Never

    private let queue: SKPaymentQueue

    init(queue: SKPaymentQueue) {
        self.queue = queue
    }

    func receive<S>(subscriber: S) where S : Subscriber, Failure == S.Failure, Output == S.Input {
        let subscription = SKPaymentQueueSubscription(
            queue: queue, subscriber: subscriber)
        subscriber.receive(subscription: subscription)
    }

}

extension SKPaymentQueue {
    var publisher: SKPaymentQueuePublisher {
        return .init(queue: self)
    }
}

/// A subscription implementation for the given `SKPaymentQueue`
private class SKPaymentQueueSubscription: NSObject, Subscription, SKPaymentTransactionObserver {
    private let queue: SKPaymentQueue
    private let subscriber: AnySubscriber<SKPaymentTransaction, Never>

    init<S>(queue: SKPaymentQueue, subscriber: S)
        where S: Subscriber, S.Failure == Never, S.Input == SKPaymentTransaction
    {
        self.queue = queue
        self.subscriber = AnySubscriber(subscriber)

        super.init()

        queue.add(self)
    }

    func request(_ demand: Subscribers.Demand) {
        // no-op
    }

    func cancel() {
        queue.remove(self)
    }

    // MARK: - SKPaymentTransactionObserver

    func paymentQueue(_ queue: SKPaymentQueue, updatedTransactions transactions: [SKPaymentTransaction]) {
        for transaction in transactions {
            _ = subscriber.receive(transaction)
        }
    }

}
