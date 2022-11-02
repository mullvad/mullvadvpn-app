//
//  RESTRetryStrategy.swift
//  MullvadREST
//
//  Created by pronebird on 09/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    public enum WaitStrategy {
        case constant(DispatchTimeInterval)
        case exponential(base: TimeInterval, multiplier: Double)

        public var iterator: AnyIterator<DispatchTimeInterval> {
            switch self {
            case let .constant(timeInterval):
                return IntervalIterator.constant(timeInterval)
            case let .exponential(base, multiplier):
                return IntervalIterator.exponential(base: base, multiplier: multiplier)
            }
        }
    }

    public struct RetryStrategy {
        public var maxRetryCount: Int
        public var retryDelay: WaitStrategy

        /// Strategy configured to never retry.
        public static var noRetry = RetryStrategy(maxRetryCount: 0, retryDelay: .constant(.never))

        /// Startegy configured with 3 retry attempts with 2 seconds delay between.
        public static var `default` = RetryStrategy(
            maxRetryCount: 3,
            retryDelay: .constant(.seconds(2))
        )

        /// Strategy configured with 10 retry attempts with 2 seconds delay between.
        public static var aggressive = RetryStrategy(
            maxRetryCount: 10,
            retryDelay: .constant(.seconds(2))
        )

        public static var exponentialBackoffRecursiveDelay = RetryStrategy(
            maxRetryCount: 12,
            retryDelay: .exponential(base: 2, multiplier: 2)
        )
    }
}
