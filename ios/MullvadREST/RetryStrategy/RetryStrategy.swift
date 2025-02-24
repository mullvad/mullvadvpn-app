//
//  RetryStrategy.swift
//  MullvadREST
//
//  Created by pronebird on 09/12/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntime
import MullvadTypes

extension REST {
    public struct RetryStrategy: Sendable {
        public var maxRetryCount: Int
        public var delay: RetryDelay
        public var applyJitter: Bool

        public init(maxRetryCount: Int, delay: RetryDelay, applyJitter: Bool) {
            self.maxRetryCount = maxRetryCount
            self.delay = delay
            self.applyJitter = applyJitter
        }

        /// The return value of this function *must* be passed to a Rust FFI function that will consume it, otherwise it will leak.
        public func toRustStrategy() -> SwiftRetryStrategy {
            switch delay {
            case .never:
                return mullvad_api_retry_strategy_never()
            case let .constant(duration):
                return mullvad_api_retry_strategy_constant(UInt(maxRetryCount), UInt64(duration.seconds))
            case let .exponentialBackoff(initial, multiplier, maxDelay):
                return mullvad_api_retry_strategy_exponential(
                    UInt(maxRetryCount),
                    UInt64(initial.seconds),
                    UInt32(multiplier),
                    UInt64(maxDelay.seconds)
                )
            }
        }

        public func makeDelayIterator() -> AnyIterator<Duration> {
            let inner = delay.makeIterator()

            if applyJitter {
                return switch delay {
                case .never:
                    AnyIterator(inner)
                case .constant:
                    AnyIterator(Jittered(inner))
                case let .exponentialBackoff(_, _, maxDelay):
                    AnyIterator(Transformer(inner: Jittered(inner)) { nextValue in
                        guard let nextValue else { return maxDelay }
                        return nextValue >= maxDelay ? maxDelay : nextValue
                    })
                }
            } else {
                return AnyIterator(inner)
            }
        }

        /// Strategy configured to never retry.
        public static let noRetry = RetryStrategy(
            maxRetryCount: 0,
            delay: .never,
            applyJitter: false
        )

        /// Strategy configured with 2 retry attempts and exponential backoff.
        public static let `default` = RetryStrategy(
            maxRetryCount: 2,
            delay: defaultRetryDelay,
            applyJitter: true
        )

        /// Strategy configured with 10 retry attempts and exponential backoff.
        public static let aggressive = RetryStrategy(
            maxRetryCount: 10,
            delay: defaultRetryDelay,
            applyJitter: true
        )

        /// Default retry delay.
        public static let defaultRetryDelay: RetryDelay = .exponentialBackoff(
            initial: .seconds(2),
            multiplier: 2,
            maxDelay: .seconds(8)
        )

        public static let postQuantumKeyExchange = RetryStrategy(
            maxRetryCount: 10,
            delay: .exponentialBackoff(
                initial: .seconds(10),
                multiplier: UInt64(2),
                maxDelay: .seconds(30)
            ),
            applyJitter: true
        )

        public static let failedMigrationRecovery = RetryStrategy(
            maxRetryCount: .max,
            delay: .exponentialBackoff(
                initial: .seconds(5),
                multiplier: UInt64(1),
                maxDelay: .minutes(1)
            ),
            applyJitter: true
        )
    }

    public enum RetryDelay: Equatable, Sendable {
        /// Never wait to retry.
        case never

        /// Constant delay.
        case constant(Duration)

        /// Exponential backoff.
        case exponentialBackoff(initial: Duration, multiplier: UInt64, maxDelay: Duration)

        func makeIterator() -> AnyIterator<Duration> {
            switch self {
            case .never:
                return AnyIterator {
                    nil
                }

            case let .constant(duration):
                return AnyIterator {
                    duration
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
