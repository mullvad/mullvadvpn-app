//
//  CustomDateComponentsFormatting.swift
//  MullvadVPN
//
//  Created by pronebird on 14/05/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum CustomDateComponentsFormatting {}

extension CustomDateComponentsFormatting {
    /// Format a duration between the given dates returning a string that only contains one unit.
    ///
    /// The behaviour of that method differs from `DateComponentsFormatter`:
    ///
    /// 1. Intervals of two years or more are formatted in years quantity.
    /// 2. Otherwise intervals matching none of the above are formatted in days quantity.
    ///
    static func localizedString(
        from start: Date,
        to end: Date,
        calendar: Calendar = Calendar.current,
        unitsStyle: DateComponentsFormatter.UnitsStyle
    ) -> String? {
        let formatter = DateComponentsFormatter()
        formatter.calendar = calendar
        formatter.unitsStyle = unitsStyle
        formatter.maximumUnitCount = 1

        let dateComponents = calendar.dateComponents([.year, .day], from: start, to: end)
        let years = dateComponents.year ?? 0
        let days = dateComponents.day ?? 0

        if years >= 2 {
            formatter.allowedUnits = [.year]
            return formatter.string(from: dateComponents)
        } else if days > 0 {
            formatter.allowedUnits = [.day]
            return formatter.string(from: start, to: end)
        }
        return formatter.string(from: DateComponents(calendar: calendar, day: 0))
    }
}
