//
//  ExponentialBackoff.swift
//  MullvadREST
//
//  Created by pronebird on 03/11/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

struct ExponentialBackoff: IteratorProtocol {
    private var _next: Duration
    private let multiplier: UInt64
    private let maxDelay: Duration

    init(initial: Duration, multiplier: UInt64, maxDelay: Duration) {
        _next = initial
        self.multiplier = multiplier
        self.maxDelay = maxDelay
    }

    mutating func next() -> Duration? {
        let next = _next

        if next >= maxDelay {
            return maxDelay
        }

        let nextMilliseconds = next.milliseconds
        let maxMilliseconds = maxDelay.milliseconds

        let (value, overflow) = nextMilliseconds.multipliedReportingOverflow(by: Int(multiplier))
        if overflow {
            _next = .milliseconds(maxMilliseconds)
        } else {
            _next = .milliseconds(value)
        }
        return next
    }
}
