//
//  Task+.swift
//  PacketTunnelCore
//
//  Created by pronebird on 11/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension Task where Success == Never, Failure == Never {
    /**
     Suspends the current task for at least the given duration.

     Negative durations are clamped to zero.

     - Parameter duration: duration that determines how long the task should be suspended.
     */
    static func sleep(duration: Duration) async throws {
        let millis = UInt64(max(0, duration.milliseconds))
        let nanos = millis.saturatingMultiplication(1_000_000)

        try await Task.sleep(nanoseconds: nanos)
    }
}
