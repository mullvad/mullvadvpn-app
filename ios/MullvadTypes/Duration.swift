//
//  Duration.swift
//  MullvadTypes
//
//  Created by Jon Petersson on 2023-08-16.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Custom implementation of iOS native `Duration` (available from iOS16). Meant as a
/// drop-in replacement until the app supports iOS16. Ideally this whole file can
/// then be deleted without affecting the rest of the code base.
@available(iOS, introduced: 14.0, obsoleted: 16.0, message: "Replace with native Duration type.")
public struct Duration {
    private(set) var components: (seconds: Int64, attoseconds: Int64)

    public init(secondsComponent: Int64, attosecondsComponent: Int64 = 0) {
        components = (
            seconds: Int64(secondsComponent),
            attoseconds: Int64(attosecondsComponent)
        )
    }

    public static func milliseconds(_ milliseconds: Int) -> Duration {
        let subSeconds = milliseconds % 1000
        let seconds = (milliseconds - subSeconds) / 1000

        return Duration(
            secondsComponent: Int64(seconds),
            attosecondsComponent: Int64(subSeconds) * Int64(1e15)
        )
    }

    public static func seconds(_ seconds: Int) -> Duration {
        return Duration(secondsComponent: Int64(seconds))
    }

    public func logFormat() -> String {
        let timeInterval = timeInterval

        guard timeInterval >= 1 else {
            return "\(milliseconds)ms"
        }

        let trailingZeroesSuffix = ".00"
        var string = String(format: "%.2f", timeInterval)

        if string.hasSuffix(trailingZeroesSuffix) {
            string.removeLast(trailingZeroesSuffix.count)
        }

        return "\(string)s"
    }
}

extension Duration: DurationProtocol {
    public static var zero: Duration {
        return .seconds(0)
    }

    public static func / (lhs: Duration, rhs: Int) -> Duration {
        return .milliseconds(lhs.milliseconds / max(rhs, 1))
    }

    public static func * (lhs: Duration, rhs: Int) -> Duration {
        return .milliseconds(lhs.milliseconds.saturatingMultiplication(rhs))
    }

    public static func / (lhs: Duration, rhs: Duration) -> Double {
        guard rhs != .zero else {
            return lhs.timeInterval
        }

        return lhs.timeInterval / rhs.timeInterval
    }

    public static func + (lhs: Duration, rhs: Duration) -> Duration {
        return .milliseconds(lhs.milliseconds.saturatingAddition(rhs.milliseconds))
    }

    public static func - (lhs: Duration, rhs: Duration) -> Duration {
        return .milliseconds(lhs.milliseconds.saturatingSubtraction(rhs.milliseconds))
    }

    public static func < (lhs: Duration, rhs: Duration) -> Bool {
        return lhs.timeInterval < rhs.timeInterval
    }

    public static func == (lhs: Duration, rhs: Duration) -> Bool {
        return lhs.timeInterval == rhs.timeInterval
    }
}
