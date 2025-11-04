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

        let nextNanoseconds = next.totalNanoseconds
        let maxNanoseconds = maxDelay.totalNanoseconds

        let multiplied = nextNanoseconds * Double(multiplier)
        let value = min(multiplied, maxNanoseconds)

        _next = .nanoseconds(Int64(value.rounded()))
        return next
    }
}

extension Duration {
    var totalNanoseconds: Double {
        let c = components
        return Double(c.seconds) * 1_000_000_000
            + Double(c.attoseconds) / 1_000_000_000.0
    }
}
