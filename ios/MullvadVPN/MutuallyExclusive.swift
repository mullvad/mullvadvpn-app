//
//  MutuallyExclusive.swift
//  MullvadVPN
//
//  Created by pronebird on 24/10/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Combine
import Foundation

extension Publishers {

    /// A publisher that blocks the given DispatchQueue until the produced publisher reported the
    /// completion.
    final class MutuallyExclusive<PublisherType, Context>: Publisher where PublisherType: Publisher, Context: Scheduler {

        typealias Output = PublisherType.Output
        typealias Failure = PublisherType.Failure

        typealias MakePublisherBlock = () -> PublisherType

        private let exclusivityQueue: Context
        private let executionQueue: Context

        private let createPublisher: MakePublisherBlock

        init(exclusivityQueue: Context, executionQueue: Context, createPublisher: @escaping MakePublisherBlock) {
            self.exclusivityQueue = exclusivityQueue
            self.executionQueue = executionQueue
            self.createPublisher = createPublisher
        }

        func receive<S>(subscriber: S) where S : Subscriber, S.Failure == Failure, S.Input == Output {
            exclusivityQueue.schedule {
                let sema = DispatchSemaphore(value: 0)
                let releaseLock = {
                    _ = sema.signal()
                }

                self.executionQueue.schedule {
                    self.createPublisher()
                        .handleEvents(receiveCompletion: { _ in
                            releaseLock()
                        }, receiveCancel: {
                            releaseLock()
                        })
                        .subscribe(subscriber)
                }

                sema.wait()
            }
        }
    }

}

typealias MutuallyExclusive = Publishers.MutuallyExclusive
