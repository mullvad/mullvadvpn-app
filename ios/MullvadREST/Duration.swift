//
//  Duration.swift
//  MullvadREST
//
//  Created by pronebird on 04/11/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    public struct Duration: Comparable {
        public let milliseconds: UInt64

        public var seconds: UInt64 {
            milliseconds / 1000
        }

        public var timeInterval: TimeInterval {
            TimeInterval(milliseconds) / 1000
        }

        private init(milliseconds: UInt64) {
            self.milliseconds = milliseconds
        }

        public func format() -> String {
            guard milliseconds >= 1000 else {
                return "\(milliseconds)ms"
            }

            let trailingZeroesSuffix = ".00"
            var string = String(format: "%.2f", timeInterval)

            if string.hasSuffix(trailingZeroesSuffix) {
                string.removeLast(trailingZeroesSuffix.count)
            }

            return "\(string)s"
        }

        public static func seconds(_ seconds: UInt64) -> Duration {
            Duration(milliseconds: seconds.saturatingMultiplication(1000))
        }

        public static func milliseconds(_ milliseconds: UInt64) -> Duration {
            Duration(milliseconds: milliseconds)
        }

        public static func < (lhs: Duration, rhs: Duration) -> Bool {
            lhs.milliseconds < rhs.milliseconds
        }

        public static func == (lhs: Duration, rhs: Duration) -> Bool {
            lhs.milliseconds == rhs.milliseconds
        }

        public static func * (lhs: Duration, factor: UInt64) -> Duration {
            Duration(milliseconds: lhs.milliseconds.saturatingMultiplication(factor))
        }
    }
}
