//
//  CancellableDelayPublisher.swift
//  MullvadVPN
//
//  Created by pronebird on 02/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Combine

extension Publisher {

    /// This is a `delay` operator implementation that respects cancellation
    func cancellableDelay<S>(for delay: S.SchedulerTimeType.Stride, scheduler: S)
        -> Publishers.FlatMap<PassthroughSubject<Self.Output, Self.Failure>, Self>
        where S: Scheduler
    {
        return self.flatMap { (value) -> PassthroughSubject<Output, Failure> in
            let subject = PassthroughSubject<Output, Failure>()
            let date = scheduler.now.advanced(by: delay)

            // `PassthroughSubject` does not emit values, nor completion after cancellation
            scheduler.schedule(after: date) {
                subject.send(value)
                subject.send(completion: .finished)
            }

            return subject
        }
    }
}
