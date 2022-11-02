//
//  WaitStrategyIntervalIterator.swift
//  MullvadREST
//
//  Created by Sajad Vishkai on 2022-11-01.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST.WaitStrategy {
    private struct CountingIterator<Element>: IteratorProtocol {
        private let body: (Int) -> Element?
        public private(set) var count = 0

        public init(_ body: @escaping (Int) -> Element?) {
            self.body = body
        }

        public mutating func next() -> Element? {
            defer { count = count + 1 }
            return body(count)
        }
    }

    internal enum IntervalIterator {
        public static func constant(_ constant: DispatchTimeInterval)
            -> AnyIterator<DispatchTimeInterval>
        {
            return AnyIterator { constant }
        }

        public static func exponential(
            base: TimeInterval = 1.0,
            multiplier: Double = 2.0
        ) -> AnyIterator<DispatchTimeInterval> {
            return AnyIterator(CountingIterator { count in
                let interval = base * pow(multiplier, Double(count))
                return .seconds(
                    Int(interval)
                )
            })
        }
    }
}
