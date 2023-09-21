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

     Negative duratons are clamped to zero.

     - Parameter duration: duration that determines how long the task should be suspended.
     */
    static func sleep(duration: Duration) async throws {
        let nanoseconds = UInt64(max(0, duration.nanosecondsClamped))

        try await Task.sleep(nanoseconds: nanoseconds)
    }
}

private extension Duration {
    /// The duration represented as nanoseconds, clamped to maximum expressible value.
    var nanosecondsClamped: Int64 {
        let secondsNanos = components.seconds.multipliedReportingOverflow(by: 1_000_000_000)
        guard !secondsNanos.overflow else { return .max }

        let attosNanos = components.attoseconds.dividedReportingOverflow(by: 1_000_000_000)
        guard !attosNanos.overflow else { return .max }

        let combinedNanos = secondsNanos.partialValue.addingReportingOverflow(attosNanos.partialValue)
        guard !combinedNanos.overflow else { return .max }

        return combinedNanos.partialValue
    }
}
