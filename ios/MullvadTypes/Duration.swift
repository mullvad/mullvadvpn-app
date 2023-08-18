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
public struct Duration {
    private(set) var components: (seconds: Int64, attoseconds: Int64)

    public init(secondsComponent: Int64, attosecondsComponent: Int64 = 0) {
        components = (
            seconds: Int64(secondsComponent),
            attoseconds: Int64(attosecondsComponent) * Int64(1e15)
        )
    }

    public static func milliseconds(_ milliseconds: Int) -> Duration {
        return duration(from: TimeInterval(milliseconds / 1000))
    }

    public static func seconds(_ seconds: Int) -> Duration {
        return duration(from: TimeInterval(seconds))
    }

    public func formatted() -> String {
        let timeInterval = timeInterval

        guard timeInterval >= 1 else {
            let milliseconds = Int(timeInterval.truncatingRemainder(dividingBy: 1000))
            return "\(milliseconds) ms"
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
        return duration(from: lhs.timeInterval / Double(rhs))
    }

    public static func * (lhs: Duration, rhs: Int) -> Duration {
        return duration(from: lhs.timeInterval * Double(rhs))
    }

    public static func / (lhs: Duration, rhs: Duration) -> Double {
        return lhs.timeInterval / rhs.timeInterval
    }

    public static func + (lhs: Duration, rhs: Duration) -> Duration {
        return duration(from: lhs.timeInterval + rhs.timeInterval)
    }

    public static func - (lhs: Duration, rhs: Duration) -> Duration {
        return duration(from: lhs.timeInterval - rhs.timeInterval)
    }

    public static func < (lhs: Duration, rhs: Duration) -> Bool {
        return lhs.timeInterval < rhs.timeInterval
    }

    public static func == (lhs: Duration, rhs: Duration) -> Bool {
        return lhs.timeInterval == rhs.timeInterval
    }
}
