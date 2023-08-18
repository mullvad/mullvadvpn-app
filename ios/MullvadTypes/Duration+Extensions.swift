//
//  Duration+Extensions.swift
//  MullvadTypes
//
//  Created by Jon Petersson on 2023-08-18.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

// Extends Duration with convenience accessors and functions.
extension Duration {
    public var isFinite: Bool {
        return timeInterval.isFinite
    }

    public var timeInterval: TimeInterval {
        return Double(components.seconds) + (Double(components.attoseconds) * 1e-18)
    }

    public static func minutes(_ minutes: Int) -> Duration {
        return .seconds(minutes * 60)
    }

    public static func hours(_ hours: Int) -> Duration {
        return .seconds(hours * 3600)
    }

    public static func days(_ days: Int) -> Duration {
        return .seconds(days * 86400)
    }

    public static func duration(from timeInterval: TimeInterval) -> Duration {
        let milliseconds = (timeInterval * 1000).truncatingRemainder(dividingBy: 1000)

        return Duration(
            secondsComponent: Int64(timeInterval),
            attosecondsComponent: Int64(milliseconds) * Int64(1e15)
        )
    }
}

// Extends Duration with custom operators.
extension Duration {
    public static func + (lhs: DispatchWallTime, rhs: Duration) -> DispatchWallTime {
        return lhs + rhs.timeInterval
    }

    public static func * (lhs: Duration, rhs: TimeInterval) -> Duration {
        return duration(from: lhs.timeInterval * rhs)
    }

    public static func + (lhs: Duration, rhs: TimeInterval) -> Duration {
        return duration(from: lhs.timeInterval + rhs)
    }

    public static func - (lhs: Duration, rhs: TimeInterval) -> Duration {
        return duration(from: lhs.timeInterval - rhs)
    }

    public static func >= (lhs: TimeInterval, rhs: Duration) -> Bool {
        return lhs >= rhs.timeInterval
    }

    public static func <= (lhs: TimeInterval, rhs: Duration) -> Bool {
        return lhs <= rhs.timeInterval
    }

    public static func < (lhs: TimeInterval, rhs: Duration) -> Bool {
        return lhs <= rhs.timeInterval
    }

    public static func > (lhs: TimeInterval, rhs: Duration) -> Bool {
        return lhs <= rhs.timeInterval
    }
}
