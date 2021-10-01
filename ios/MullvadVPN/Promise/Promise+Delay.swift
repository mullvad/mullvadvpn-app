//
//  Promise+Delay.swift
//  Promise+Delay
//
//  Created by pronebird on 07/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Promise {
    /// Delay observing the upstream by the given interval.
    func delay(by timeInterval: DispatchTimeInterval, timerType: TimerType, queue: DispatchQueue? = nil) -> Promise<Value> {
        return Promise<Value>(parent: self) { resolver in
            let timer = DispatchSource.makeTimerSource(flags: [], queue: queue)

            let timerCancelHandler = DispatchWorkItem {
                resolver.resolve(completion: .cancelled, queue: queue)
            }

            timer.setEventHandler {
                // Prevent potential further invocation of cancel handler
                timerCancelHandler.cancel()

                self.observe { completion in
                    resolver.resolve(completion: completion, queue: queue)
                }
            }

            timer.setCancelHandler(handler: timerCancelHandler)

            resolver.setCancelHandler {
                timer.cancel()
            }

            switch timerType {
            case .deadline:
                timer.schedule(deadline: .now() + timeInterval)
            case .walltime:
                timer.schedule(wallDeadline: .now() + timeInterval)
            }

            timer.activate()
        }
    }
}
