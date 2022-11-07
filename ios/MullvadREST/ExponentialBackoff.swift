//
//  ExponentialBackoff.swift
//  MullvadREST
//
//  Created by pronebird on 03/11/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

struct ExponentialBackoff: IteratorProtocol {
    private var _next: REST.Duration
    private let multiplier: UInt64
    private let maxDelay: REST.Duration?

    init(initial: REST.Duration, multiplier: UInt64, maxDelay: REST.Duration? = nil) {
        _next = initial
        self.multiplier = multiplier
        self.maxDelay = maxDelay
    }

    mutating func next() -> REST.Duration? {
        let next = _next

        if let maxDelay = maxDelay, next > maxDelay {
            return maxDelay
        }

        _next = next * multiplier

        return next
    }
}

struct Jittered<InnerIterator: IteratorProtocol>: IteratorProtocol
    where InnerIterator.Element == REST.Duration
{
    private var inner: InnerIterator

    init(_ inner: InnerIterator) {
        self.inner = inner
    }

    mutating func next() -> REST.Duration? {
        guard let interval = inner.next() else { return nil }

        let jitter = Double.random(in: 0.0 ... 1.0)
        let millis = interval.milliseconds
        let millisWithJitter = millis.saturatingAddition(UInt64(Double(millis) * jitter))

        return .milliseconds(millisWithJitter)
    }
}
