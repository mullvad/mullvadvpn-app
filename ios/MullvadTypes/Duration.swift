//
//  Duration.swift
//  MullvadTypes
//
//  Created by Jon Petersson on 2023-08-16.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct Duration {
    private var components: (seconds: Int64, attoseconds: Int64)

    private init(seconds: Int, milliseconds: Double = 0) {
        components = (
            seconds: Int64(seconds),
            attoseconds: Int64(milliseconds) * Int64(1e15)
        )
    }

    public static func milliseconds(_ milliseconds: Int) -> Duration {
        return Duration(
            seconds: milliseconds / 1000,
            milliseconds: Double(milliseconds).truncatingRemainder(dividingBy: 1000)
        )
    }

    public static func seconds(_ seconds: Int) -> Duration {
        return Duration(seconds: seconds)
    }
}

extension Duration {
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

    public static func + (rhs: DispatchWallTime, lhs: Duration) -> DispatchWallTime {
        return rhs + lhs.timeInterval
    }
}
