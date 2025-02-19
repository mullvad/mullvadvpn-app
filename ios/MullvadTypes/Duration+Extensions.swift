//
//  Duration+Extensions.swift
//  MullvadTypes
//
//  Created by Jon Petersson on 2023-08-18.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

// Extends Duration with convenience accessors and functions.
extension Duration {
    public var isFinite: Bool {
        return timeInterval.isFinite
    }
    
    public var seconds: Int64 {
        return components.seconds
    }
    
    public var timeInterval: TimeInterval {
        return TimeInterval(components.seconds) + (TimeInterval(components.attoseconds) * 1e-18)
    }

    public var milliseconds: Int {
        return Int(components.seconds.saturatingMultiplication(1000)) + Int(Double(components.attoseconds) * 1e-15)
    }

    public static func minutes(_ minutes: Int) -> Duration {
        return .seconds(minutes.saturatingMultiplication(60))
    }

    public static func hours(_ hours: Int) -> Duration {
        return .seconds(hours.saturatingMultiplication(3600))
    }

    public static func days(_ days: Int) -> Duration {
        return .seconds(days.saturatingMultiplication(86400))
    }
}

// Extends Duration with custom operators.
extension Duration {
    public static func + (lhs: DispatchWallTime, rhs: Duration) -> DispatchWallTime {
        return lhs + rhs.timeInterval
    }

    public static func * (lhs: Duration, rhs: Double) -> Duration {
        let milliseconds = lhs.timeInterval * rhs * 1000

        let maxTruncated = min(milliseconds, Double(Int.max).nextDown)
        let bothTruncated = max(maxTruncated, Double(Int.min).nextUp)

        return .milliseconds(Int(bothTruncated))
    }

    public static func + (lhs: Duration, rhs: TimeInterval) -> Duration {
        return .milliseconds(lhs.milliseconds.saturatingAddition(Int(rhs * 1000)))
    }

    public static func - (lhs: Duration, rhs: TimeInterval) -> Duration {
        return .milliseconds(lhs.milliseconds.saturatingSubtraction(Int(rhs * 1000)))
    }

    public static func >= (lhs: TimeInterval, rhs: Duration) -> Bool {
        return lhs >= rhs.timeInterval
    }

    public static func <= (lhs: TimeInterval, rhs: Duration) -> Bool {
        return lhs <= rhs.timeInterval
    }

    public static func < (lhs: TimeInterval, rhs: Duration) -> Bool {
        return lhs < rhs.timeInterval
    }

    public static func > (lhs: TimeInterval, rhs: Duration) -> Bool {
        return lhs > rhs.timeInterval
    }
}
