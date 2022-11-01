//
//  RESTRetryStrategy.swift
//  MullvadREST
//
//  Created by pronebird on 09/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    struct ExponentialBackoff: IteratorProtocol {
        let base: TimeInterval
        let multiplier: Int

        private var i = 0

        init(base: TimeInterval, multiplier: Int) {
            self.base = base
            self.multiplier = multiplier
        }

        mutating func next() -> DispatchTimeInterval? {
            defer { i += 1 }

            return .seconds(
                Int(base * pow(Double(multiplier), Double(i)))
            )
        }
    }

    struct Jittered<I: IteratorProtocol>: IteratorProtocol where I.Element == DispatchTimeInterval {
        private var inner: I

        init(_ inner: I) {
            self.inner = inner
        }

        mutating func next() -> DispatchTimeInterval? {
            guard let random = (1 ... 256).randomElement(),
                  case let .seconds(interval) = inner.next() else { return nil }

            let jitter = random / 256

            return .seconds(
                Int(interval + interval * jitter)
            )
        }
    }

    public enum WaitStrategy {
        case constant(DispatchTimeInterval)
        case exponential(base: TimeInterval, multiplier: Double, jittered: Bool)

        public var iterator: AnyIterator<DispatchTimeInterval> {
            switch self {
            case let .constant(timeInterval):
                return AnyIterator { timeInterval }

            case let .exponential(base, multiplier, jittered):
                let inner = ExponentialBackoff(base: base, multiplier: Int(multiplier))
                return jittered ? AnyIterator(Jittered(inner)) : AnyIterator(inner)
            }
        }
    }

    public struct RetryStrategy {
        public var maxRetryCount: Int
        public var retryDelay: WaitStrategy
        public var jitter: Bool

        /// Strategy configured to never retry.
        public static var noRetry = RetryStrategy(
            maxRetryCount: 0,
            retryDelay: .constant(.never),
            jitter: false
        )

        /// Startegy configured with 3 retry attempts with 2 seconds delay between.
        public static var `default` = RetryStrategy(
            maxRetryCount: 3,
            retryDelay: .constant(.seconds(2)),
            jitter: false
        )

        /// Strategy configured with 10 retry attempts with 2 seconds delay between.
        public static var aggressive = RetryStrategy(
            maxRetryCount: 10,
            retryDelay: .constant(.seconds(2)),
            jitter: false
        )
    }
}
