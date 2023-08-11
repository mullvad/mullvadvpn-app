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
    private var _next: Duration
    private let multiplier: UInt64
    private let maxDelay: Duration?

    init(initial: Duration, multiplier: UInt64, maxDelay: Duration? = nil) {
        _next = initial
        self.multiplier = multiplier
        self.maxDelay = maxDelay
    }

    mutating func next() -> Duration? {
        let next = _next

        if let maxDelay, next > maxDelay {
            return maxDelay
        }

        _next = next * Int(multiplier)

        return next
    }
}

struct Jittered<InnerIterator: IteratorProtocol>: IteratorProtocol
    where InnerIterator.Element == Duration {
    private var inner: InnerIterator

    init(_ inner: InnerIterator) {
        self.inner = inner
    }

    mutating func next() -> Duration? {
        guard let interval = inner.next() else { return nil }

        let jitter = Double.random(in: 0.0 ... 1.0)
        let millis = interval.milliseconds
        let millisWithJitter = millis.saturatingAddition(Int(Double(millis) * jitter))

        return .milliseconds(millisWithJitter)
    }
}
