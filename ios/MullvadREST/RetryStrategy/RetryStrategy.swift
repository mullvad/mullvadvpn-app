// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadRustRuntime
import MullvadTypes

extension REST {
    public struct RetryStrategy: Codable, Sendable {
        public var maxRetryCount: Int
        public var delay: RetryDelay
        public var applyJitter: Bool

        public init(maxRetryCount: Int, delay: RetryDelay, applyJitter: Bool) {
            self.maxRetryCount = maxRetryCount
            self.delay = delay
            self.applyJitter = applyJitter
        }

        public func toRustStrategy() -> MullvadRustRuntime.RetryStrategy {
            switch delay {
            case .never:
                return mullvadApiRetryStrategyNever()
            case let .constant(duration):
                return mullvadApiRetryStrategyConstant(
                    maxRetries: UInt64(maxRetryCount),
                    delaySec: UInt64(duration.seconds))
            case let .exponentialBackoff(initial, multiplier, maxDelay):
                return mullvadApiRetryStrategyExponential(
                    maxRetries: UInt64(maxRetryCount),
                    initialSec: UInt64(initial.seconds),
                    factor: UInt32(multiplier),
                    maxDelaySec: UInt64(maxDelay.seconds))
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
                    AnyIterator(
                        Transformer(inner: Jittered(inner)) { nextValue in
                            let maxDelay = maxDelay.duration

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
            delay: .default,
            applyJitter: true
        )

        /// Strategy configured with 10 retry attempts and exponential backoff.
        public static let aggressive = RetryStrategy(
            maxRetryCount: 10,
            delay: .default,
            applyJitter: true
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

        public static let purchaseReceiptUpload = RetryStrategy(
            maxRetryCount: 3,
            delay: .default,
            applyJitter: true
        )
    }

    public enum RetryDelay: Codable, Equatable, Sendable {
        /// Never wait to retry.
        case never

        /// Constant delay.
        case constant(CodableDuration)

        /// Exponential backoff.
        case exponentialBackoff(initial: CodableDuration, multiplier: UInt64, maxDelay: CodableDuration)

        func makeIterator() -> AnyIterator<Duration> {
            switch self {
            case .never:
                return AnyIterator {
                    nil
                }

            case let .constant(duration):
                return AnyIterator {
                    duration.duration
                }

            case let .exponentialBackoff(initial, multiplier, maxDelay):
                return AnyIterator(
                    ExponentialBackoff(
                        initial: initial.duration,
                        multiplier: multiplier,
                        maxDelay: maxDelay.duration
                    ))
            }
        }

        /// Default retry delay.
        public static let `default`: RetryDelay = .exponentialBackoff(
            initial: .seconds(2),
            multiplier: 2,
            maxDelay: .seconds(8)
        )
    }

    public struct CodableDuration: Codable, Equatable, Sendable {
        public var seconds: Int64
        public var attoseconds: Int64

        public var duration: Duration {
            Duration(secondsComponent: seconds, attosecondsComponent: attoseconds)
        }

        public static func seconds(_ seconds: Int) -> CodableDuration {
            return CodableDuration(seconds: Int64(seconds), attoseconds: 0)
        }

        public static func minutes(_ minutes: Int) -> CodableDuration {
            return .seconds(minutes.saturatingMultiplication(60))
        }
    }
}
