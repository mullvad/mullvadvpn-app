//
//  ExponentialBackoffRetryStrategy.swift
//  MullvadREST
//
//  Created by Sajad Vishkai on 2022-11-01.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST.RetryStrategy {
    public struct ExponentialBackoff: IteratorProtocol {
        public let base: TimeInterval
        public let multiplier: Int

        private var i = 0

        public init(base: TimeInterval, multiplier: Int) {
            self.base = base
            self.multiplier = multiplier
        }

        public mutating func next() -> DispatchTimeInterval? {
            defer { i += 1 }

            return .seconds(
                Int(base * pow(Double(multiplier), Double(i)))
            )
        }
    }
}
