//
//  CancellableDelayPublisher.swift
//  MullvadVPN
//
//  Created by pronebird on 02/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Combine

extension Publishers {

    class CancellableDelay<Upstream, Context>: Publisher where Upstream: Publisher, Context: Scheduler
    {
        typealias Output = Upstream.Output
        typealias Failure = Upstream.Failure

        private let scheduler: Context
        private let delay: Context.SchedulerTimeType.Stride

        private let upstream: Upstream

        init(upstream: Upstream, scheduler: Context, delay: Context.SchedulerTimeType.Stride) {
            self.scheduler = scheduler
            self.delay = delay
            self.upstream = upstream
        }

        func receive<S>(subscriber: S) where S : Subscriber, Failure == S.Failure, Output == S.Input {
            let subscription = Subscription(
                upstream: upstream,
                downstream: subscriber,
                scheduler: scheduler,
                delay: delay)

            subscriber.receive(subscription: subscription)
        }
    }

}

private extension Publishers.CancellableDelay {

    class Subscription<Upstream, Downstream, Context>: Combine.Subscription, Subscriber
        where
        Upstream: Publisher,
        Downstream: Subscriber,
        Context: Scheduler,
        Upstream.Output == Downstream.Input,
        Upstream.Failure == Downstream.Failure
    {
        typealias Input = Downstream.Input
        typealias Failure = Downstream.Failure

        private let upstream: Upstream
        private let downstream: Downstream

        private let cancelLock = NSRecursiveLock()
        private let scheduler: Context
        private let delay: Context.SchedulerTimeType.Stride
        private var demand: Subscribers.Demand = .unlimited
        private var isCancelled = false
        private var innerSubscription: Combine.Subscription?

        init(upstream: Upstream, downstream: Downstream, scheduler: Context, delay: Context.SchedulerTimeType.Stride) {
            self.upstream = upstream
            self.downstream = downstream
            self.scheduler = scheduler
            self.delay = delay
        }

        func request(_ demand: Subscribers.Demand) {
            cancelLock.withCriticalBlock {
                guard !self.isCancelled else { return }

                self.demand = demand
                self.upstream.subscribe(self)
            }
        }

        func receive(_ input: Input) -> Subscribers.Demand {
            return self.cancelLock.withCriticalBlock { () -> Subscribers.Demand in
                delay { [weak self] in
                    _ = self?.downstream.receive(input)
                }

                // Expects the demand to decrease linearly
                self.demand -= 1

                return self.demand
            }
        }

        func receive(completion: Subscribers.Completion<Failure>) {
            delay { [weak self] in
                self?.downstream.receive(completion: completion)
            }
        }

        func receive(subscription: Combine.Subscription) {
            self.cancelLock.withCriticalBlock {
                guard !self.isCancelled else { return }

                subscription.request(self.demand)

                self.innerSubscription = subscription
            }
        }

        func cancel() {
            cancelLock.withCriticalBlock {
                isCancelled = true

                innerSubscription?.cancel()
            }
        }

        private func delay(_ action: @escaping () -> Void) {
            let date = self.scheduler.now.advanced(by: self.delay)

            self.scheduler.schedule(after: date) { [weak self] in
                guard let self = self else { return }

                self.cancelLock.withCriticalBlock {
                    if !self.isCancelled {
                        action()
                    }
                }
            }
        }

    }
}

extension Publisher {

    func cancellableDelay<S>(for delay: S.SchedulerTimeType.Stride, scheduler: S)
        -> Publishers.CancellableDelay<Self, S> where S: Scheduler
    {
        return Publishers.CancellableDelay(upstream: self, scheduler: scheduler, delay: delay)
    }

}
