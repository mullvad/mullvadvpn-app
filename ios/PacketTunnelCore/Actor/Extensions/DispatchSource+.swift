//
//  Timer+.swift
//  PacketTunnel
//
//  Created by pronebird on 31/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension DispatchSource {
    static func scheduledTimer(on startTime: DispatchWallTime, repeating: DispatchTimeInterval) -> AsyncStream<Date> {
        return AsyncStream(bufferingPolicy: .bufferingNewest(1)) { continuation in
            let timer = DispatchSource.makeTimerSource()

            timer.setEventHandler {
                continuation.yield(Date())
            }

            continuation.onTermination = { _ in
                timer.cancel()
            }

            timer.schedule(wallDeadline: startTime, repeating: repeating)
            timer.activate()
        }
    }
}
