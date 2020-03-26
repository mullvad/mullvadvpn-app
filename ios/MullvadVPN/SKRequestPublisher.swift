//
//  SKRequestPublisher.swift
//  MullvadVPN
//
//  Created by pronebird on 11/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import StoreKit

/// A protocol that formalizes the interface of all of the subclasses of
/// `SKRequestSubscription<Output>`
protocol SKRequestSubscriptionProtocol: Subscription {
    associatedtype Output
    associatedtype Failure: Error

    init<S: Subscriber>(request: SKRequest, subscriber: S)
        where S.Input == Output, S.Failure == Failure
}

/// A base implementation of subscription that handles the `SKRequest`.
class SKRequestSubscription<Output>: NSObject, Subscription, SKRequestDelegate,
    SKRequestSubscriptionProtocol
{
    typealias Failure = Error

    private let request: SKRequest
    fileprivate let subscriber: AnySubscriber<Output, Failure>

    required init<S: Subscriber>(request: SKRequest, subscriber: S)
        where S.Input == Output, S.Failure == Failure
    {
        self.request = request
        self.subscriber = AnySubscriber(subscriber)

        super.init()
        request.delegate = self
    }

    func request(_ demand: Subscribers.Demand) {
        request.start()
    }

    func cancel() {
        request.cancel()
    }

    // MARK: - SKRequestDelegate

    func request(_ request: SKRequest, didFailWithError error: Error) {
        subscriber.receive(completion: .failure(error))
    }

    func requestDidFinish(_ request: SKRequest) {
        subscriber.receive(completion: .finished)
    }
}

/// A subscription that emits the `SKProductsResponse` upon request completion
class SKProductsRequestSubscription: SKRequestSubscription<SKProductsResponse>,
    SKProductsRequestDelegate
{

    // MARK: - SKProductsRequestDelegate

    func productsRequest(_ request: SKProductsRequest, didReceive response: SKProductsResponse) {
        _ = self.subscriber.receive(response)
    }

}

/// A subscription for requesting the AppStore receipt refresh
class SKRefreshRequestSubscription: SKRequestSubscription<()> {
    override func requestDidFinish(_ request: SKRequest) {
        // Emit void so that publishers using this subscription could be chained
        _ = self.subscriber.receive(())

        super.requestDidFinish(request)
    }
}

/// A base implementation of publisher that runs `SKRequest`s
class SKRequestPublisher<SubscriptionType>: Publisher
    where SubscriptionType: SKRequestSubscriptionProtocol
{
    typealias Output = SubscriptionType.Output
    typealias Failure = SubscriptionType.Failure

    fileprivate let request: SKRequest

    init(request: SKRequest) {
        self.request = request
    }

    func receive<S>(subscriber: S) where S : Subscriber, Failure == S.Failure, Output == S.Input {
        let subscription = SubscriptionType(request: request, subscriber: subscriber)

        subscriber.receive(subscription: subscription)
    }

}

protocol SKRequestPublishing {
    associatedtype SubscriptionType: SKRequestSubscriptionProtocol

    var publisher: SKRequestPublisher<SubscriptionType> { get }
}

extension SKProductsRequest: SKRequestPublishing {
    var publisher: SKRequestPublisher<SKProductsRequestSubscription> {
        return .init(request: self)
    }
}

extension SKReceiptRefreshRequest: SKRequestPublishing {
    var publisher: SKRequestPublisher<SKRefreshRequestSubscription> {
        return .init(request: self)
    }
}
