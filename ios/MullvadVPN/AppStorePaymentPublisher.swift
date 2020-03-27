//
//  AppStorePaymentPublisher.swift
//  MullvadVPN
//
//  Created by pronebird on 23/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import StoreKit

class AppStorePaymentPublisher: Publisher {
    typealias Output = SendAppStoreReceiptResponse
    typealias Failure = AppStorePaymentManager.Error

    private let paymentManager: AppStorePaymentManager
    private let payment: SKPayment
    private let queue: SKPaymentQueue

    init(paymentManager: AppStorePaymentManager, queue: SKPaymentQueue, payment: SKPayment) {
        self.paymentManager = paymentManager
        self.payment = payment
        self.queue = queue
    }

    func receive<S>(subscriber: S) where S : Subscriber, Failure == S.Failure, Output == S.Input {
        let subscription = AppStorePaymentSubscription(
            paymentManager: paymentManager,
            queue: queue,
            payment: payment,
            subscriber: subscriber)

        subscriber.receive(subscription: subscription)
    }
}

private class AppStorePaymentSubscription: Subscription, AppStorePaymentObserver {

    typealias Output = SendAppStoreReceiptResponse
    typealias Failure = AppStorePaymentManager.Error

    private let paymentManager: AppStorePaymentManager
    private let payment: SKPayment
    private let queue: SKPaymentQueue
    private let subscriber: AnySubscriber<Output, Failure>

    init<S>(paymentManager: AppStorePaymentManager, queue: SKPaymentQueue, payment: SKPayment, subscriber: S)
        where S: Subscriber, S.Input == Output, S.Failure == Failure
    {
        self.paymentManager = paymentManager
        self.payment = payment
        self.queue = queue
        self.subscriber = AnySubscriber(subscriber)

        paymentManager.addPaymentObserver(self)
    }

    func request(_ demand: Subscribers.Demand) {
        queue.add(payment)
    }

    func cancel() {
        paymentManager.removePaymentObserver(self)
    }

    // MARK: - AppStorePaymentObserver

    func appStorePaymentManager(_ manager: AppStorePaymentManager,
                                transaction: SKPaymentTransaction,
                                didFinishWithResponse response: SendAppStoreReceiptResponse)
    {
        if transaction.payment == payment {
            _ = subscriber.receive(response)
            subscriber.receive(completion: .finished)
        }
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager,
                                transaction: SKPaymentTransaction,
                                didFailWithError error: AppStorePaymentManager.Error) {
        if transaction.payment == payment {
            subscriber.receive(completion: .failure(error))
        }
    }

}
