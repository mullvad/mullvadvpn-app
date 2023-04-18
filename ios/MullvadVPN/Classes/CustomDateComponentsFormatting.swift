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
    /// 2. Otherwise intervals matching  are formatted in days quantity.
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
        formatter.allowedUnits = [.minute, .hour, .day, .month, .year]
        formatter.maximumUnitCount = 1

        let dateComponents = calendar
            .dateComponents([.year, .day, .hour, .minute, .second], from: start, to: end)

        let years = dateComponents.year ?? 0

        if years >= 2 {
            formatter.allowedUnits = [.year]
            return formatter.string(from: dateComponents)
        } else {
            formatter.allowedUnits = [.day]
            return formatter.string(from: start, to: end)
        }
    }
}
