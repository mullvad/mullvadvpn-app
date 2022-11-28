//
//  RESTRetryStrategy.swift
//  MullvadREST
//
//  Created by pronebird on 09/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    public struct RetryStrategy {
        public var maxRetryCount: Int
        public var delay: RetryDelay
        public var applyJitter: Bool

        public init(maxRetryCount: Int, delay: RetryDelay, applyJitter: Bool) {
            self.maxRetryCount = maxRetryCount
            self.delay = delay
            self.applyJitter = applyJitter
        }

        public func makeDelayIterator() -> AnyIterator<Duration> {
            let inner = delay.makeIterator()

            if applyJitter {
                return AnyIterator(Jittered(inner))
            } else {
                return AnyIterator(inner)
            }
        }

        /// Strategy configured to never retry.
        public static var noRetry = RetryStrategy(
            maxRetryCount: 0,
            delay: .never,
            applyJitter: false
        )

        /// Startegy configured with 3 retry attempts and exponential backoff.
        public static var `default` = RetryStrategy(
            maxRetryCount: 3,
            delay: defaultRetryDelay,
            applyJitter: true
        )

        /// Strategy configured with 10 retry attempts and exponential backoff.
        public static var aggressive = RetryStrategy(
            maxRetryCount: 10,
            delay: defaultRetryDelay,
            applyJitter: true
        )

        /// Default retry delay.
        public static var defaultRetryDelay: RetryDelay = .exponentialBackoff(
            initial: .seconds(2),
            multiplier: 2,
            maxDelay: .seconds(8)
        )
    }

    public enum RetryDelay: Equatable {
        /// Never wait to retry.
        case never

        /// Constant delay.
        case constant(Duration)

        /// Exponential backoff.
        case exponentialBackoff(initial: Duration, multiplier: UInt64, maxDelay: Duration?)

        func makeIterator() -> AnyIterator<Duration> {
            switch self {
            case .never:
                return AnyIterator {
                    return nil
                }

            case let .constant(duration):
                return AnyIterator {
                    return duration
                }

            case let .exponentialBackoff(initial, multiplier, maxDelay):
                return AnyIterator(ExponentialBackoff(
                    initial: initial,
                    multiplier: multiplier,
                    maxDelay: maxDelay
                ))
            }
        }
    }
}
