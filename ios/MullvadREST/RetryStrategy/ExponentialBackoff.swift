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
