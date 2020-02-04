//
//  MutuallyExclusive.swift
//  MullvadVPN
//
//  Created by pronebird on 24/10/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation

extension Publishers {

    /// A publisher that blocks the given DispatchQueue until the produced publisher reported the
    /// completion.
    final class MutuallyExclusive<PublisherType, Context>: Publisher
        where
        PublisherType: Publisher,
        Context: Scheduler
    {
        typealias MakePublisherBlock = () -> PublisherType

        typealias Output = PublisherType.Output
        typealias Failure = PublisherType.Failure

        private let exclusivityQueue: Context
        private let executionQueue: Context

        private let createPublisher: MakePublisherBlock

        init(exclusivityQueue: Context, executionQueue: Context, createPublisher: @escaping MakePublisherBlock) {
            self.exclusivityQueue = exclusivityQueue
            self.executionQueue = executionQueue
            self.createPublisher = createPublisher
        }

        func receive<S>(subscriber: S) where S : Subscriber, S.Failure == Failure, S.Input == Output {
            let subscription = MutuallyExclusive.Subscription(
                subscriber: subscriber,
                createPublisher: createPublisher,
                exclusivityQueue: exclusivityQueue,
                executionQueue: executionQueue)

            subscriber.receive(subscription: subscription)
        }
    }
}

private extension Publishers.MutuallyExclusive {

    /// A subscription used by `MutuallyExclusive` publisher
    final class Subscription<SubscriberType, PublisherType, Context>: Combine.Subscription
        where
        SubscriberType: Subscriber, PublisherType: Publisher,
        PublisherType.Output == SubscriberType.Input,
        PublisherType.Failure == SubscriberType.Failure,
        Context: Scheduler
    {
        typealias MakePublisherBlock = () -> PublisherType

        private let subscriber: SubscriberType
        private var innerSubscriber: AnyCancellable?
        private let createPublisher: MakePublisherBlock

        private let exclusivityQueue: Context
        private let executionQueue: Context
        private let sema = DispatchSemaphore(value: 0)

        private let cancelLock = NSLock()
        private var isCancelled = false

        init(subscriber: SubscriberType,
             createPublisher: @escaping MakePublisherBlock,
             exclusivityQueue: Context,
             executionQueue: Context)
        {
            self.subscriber = subscriber
            self.createPublisher = createPublisher
            self.exclusivityQueue = exclusivityQueue
            self.executionQueue = executionQueue
        }

        func request(_ demand: Subscribers.Demand) {
            self.exclusivityQueue.schedule {
                self.executionQueue.schedule {
                    self.cancelLock.withCriticalBlock {
                        guard !self.isCancelled else { return }

                        self.innerSubscriber = self.createPublisher()
                            .sink(receiveCompletion: { [weak self] (completion) in
                                guard let self = self else { return }

                                self.subscriber.receive(completion: completion)
                                self.signalSemaphore()
                            }, receiveValue: { [weak self] (output) in
                                _ = self?.subscriber.receive(output)
                            })
                    }
                }
                self.sema.wait()
            }
        }

        func cancel() {
            cancelLock.withCriticalBlock {
                guard !isCancelled else { return }

                isCancelled = true

                innerSubscriber?.cancel()
                innerSubscriber = nil

                signalSemaphore()
            }
        }

        private func signalSemaphore() {
            _ = sema.signal()
        }

    }

}

typealias MutuallyExclusive = Publishers.MutuallyExclusive
