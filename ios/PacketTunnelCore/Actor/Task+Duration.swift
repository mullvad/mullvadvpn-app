//
//  Task+.swift
//  PacketTunnelCore
//
//  Created by pronebird on 11/09/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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
    @available(iOS, introduced: 15.0, obsoleted: 16.0, message: "Replace with Task.sleep(for:tolerance:clock:).")
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
    @available(iOS, introduced: 15.0, obsoleted: 16.0, message: "Replace with Task.sleep(for:tolerance:clock:).")
    static func sleepUsingContinuousClock(for duration: Duration) async throws {
        let timer = DispatchSource.makeTimerSource()

        try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                // The `continuation` handler should never be `resume`'d more than once.
                // Setting the eventHandler on the timer after it has been cancelled will be ignored.
                // https://github.com/apple/swift-corelibs-libdispatch/blob/77766345740dfe3075f2f60bead854b29b0cfa24/src/source.c#L338
                // Therefore, set a flag indicating `resume` has already been called to avoid `resume`ing more than once.
                // Cancelling the timer does not cancel an  event handler that is already running however,
                // the cancel handler will be scheduled after the event handler has finished.
                // https://developer.apple.com/documentation/dispatch/1385604-dispatch_source_cancel
                // Therefore, there it is safe to do this here.
                var hasResumed = false
                timer.setEventHandler {
                    hasResumed = true
                    continuation.resume()
                }
                timer.setCancelHandler {
                    guard hasResumed == false else { return }
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
