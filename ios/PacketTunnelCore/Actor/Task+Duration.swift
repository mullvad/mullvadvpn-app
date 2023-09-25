//
//  Task+.swift
//  PacketTunnelCore
//
//  Created by pronebird on 11/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

private typealias TaskCancellationError = CancellationError

extension Task where Success == Never, Failure == Never {
    /**
     Suspends the current task for at least the given duration.

     Negative durations are clamped to zero.

     - Parameter duration: duration that determines how long the task should be suspended.
     */
    @available(iOS, introduced: 14.0, obsoleted: 16.0, message: "Replace with Task.sleep(for:tolerance:clock:).")
    static func sleep(duration: Duration) async throws {
        let millis = UInt64(max(0, duration.milliseconds))
        let nanos = millis.saturatingMultiplication(1_000_000)

        try await Task.sleep(nanoseconds: nanos)
    }

    /**
     Suspends the current task for the given duration.

     Negative durations are clamped to zero.

     - Parameter duration: duration that determines how long the task should be suspended.
     */
    @available(iOS, introduced: 14.0, obsoleted: 16.0, message: "Replace with Task.sleep(for:tolerance:clock:).")
    static func sleepUsingContinuousClock(for duration: Duration) async throws {
        let timer = DispatchSource.makeTimerSource()

        try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                timer.setEventHandler {
                    continuation.resume()
                }
                timer.setCancelHandler {
                    continuation.resume(throwing: TaskCancellationError())
                }
                timer.schedule(wallDeadline: .now() + DispatchTimeInterval.milliseconds(duration.milliseconds))
                timer.activate()
            }
        } onCancel: {
            timer.cancel()
        }
    }
}
