//
//  TimeInterval+Extensions.swift
//  MullvadTypes
//
//  Created by Jon Petersson on 2023-08-11.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension TimeInterval {
    public static func milliseconds(_ milliseconds: Int) -> TimeInterval {
        return TimeInterval(milliseconds / 1000)
    }

    public static func seconds(_ seconds: Int) -> TimeInterval {
        return TimeInterval(seconds)
    }

    public static func minutes(_ minutes: Int) -> TimeInterval {
        return TimeInterval(minutes * 60)
    }

    public static func hours(_ hours: Int) -> TimeInterval {
        return TimeInterval(hours * 3600)
    }

    public static func days(_ days: Int) -> TimeInterval {
        return TimeInterval(days * 86400)
    }
}
