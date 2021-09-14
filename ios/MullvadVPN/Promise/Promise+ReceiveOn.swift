//
//  Promise+ReceiveOn.swift
//  Promise+ReceiveOn
//
//  Created by pronebird on 22/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Promise {
    /// A type of timer.
    enum TimerType {
        case deadline
        case walltime
    }

    /// Dispatch the upstream value on another queue.
    func receive(on queue: DispatchQueue) -> Promise<Value> {
        return Promise<Value> { resolver in
            self.observe { completion in
                let work = DispatchWorkItem {
                    resolver.resolve(completion: completion, queue: queue)
                }

                resolver.setCancelHandler {
                    work.cancel()
                }

                queue.async(execute: work)
            }
        }
    }

    /// Dispatch the upstream value on another queue after delay.
    func receive(on queue: DispatchQueue, after timeInterval: DispatchTimeInterval, timerType: TimerType) -> Promise<Value> {
        return Promise<Value> { resolver in
            self.observe { completion in
                let work = DispatchWorkItem {
                    resolver.resolve(completion: completion, queue: queue)
                }

                resolver.setCancelHandler {
                    work.cancel()
                }

                switch timerType {
                case .deadline:
                    queue.asyncAfter(deadline: .now() + timeInterval, execute: work)

                case .walltime:
                    queue.asyncAfter(wallDeadline: .now() + timeInterval, execute: work)
                }
            }
        }
    }
}
