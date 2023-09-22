//
//  Timer+.swift
//  PacketTunnel
//
//  Created by pronebird on 31/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension DispatchSource {
    /**
     Schedule timer and receive timer events via `AsyncStream`.

     - The stream only buffers the newest timestamp.
     - The stream is infinite for repeating timers. Cancel `AsyncStream` in order to cancel the underlying timer.

     - Parameters:
        - startTime: When the timer should fire the first event.
        - repeating: Repeat interval. The timer is considered being one-shot when `.never` is given and the stream will be finished right after the first
                     event is fired.

     - Returns: `AsyncStream` emitting `Date` at each timer invocation.
     */

    static func scheduledTimer(on startTime: DispatchWallTime, repeating: DispatchTimeInterval) -> AsyncStream<Date> {
        return AsyncStream(bufferingPolicy: .bufferingNewest(1)) { continuation in
            let timer = DispatchSource.makeTimerSource()

            timer.setEventHandler {
                continuation.yield(Date())

                if repeating == .never {
                    continuation.finish()
                }
            }

            continuation.onTermination = { _ in
                timer.cancel()
            }

            timer.schedule(wallDeadline: startTime, repeating: repeating)
            timer.activate()
        }
    }
}
