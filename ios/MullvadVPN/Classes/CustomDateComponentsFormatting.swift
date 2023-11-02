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
    /// 1. Intervals of less than a day return a custom string.
    /// 2. Intervals of two years or more are formatted in years quantity.
    /// 3. Otherwise intervals matching none of the above are formatted in days quantity.
    ///
    static func localizedString(
        from start: Date,
        to end: Date,
        calendar: Calendar = Calendar.current,
        unitsStyle: DateComponentsFormatter.UnitsStyle
    ) -> String? {
        let dateComponents = calendar.dateComponents([.year, .day], from: start, to: max(start, end))

        guard !isLessThanADayLeft(dateComponents: dateComponents) else {
            return NSLocalizedString(
                "CUSTOM_DATE_COMPONENTS_FORMATTING_LESS_THAN_ONE_DAY",
                value: "Less than a day",
                comment: ""
            )
        }

        let formatter = DateComponentsFormatter()
        formatter.calendar = calendar
        formatter.unitsStyle = unitsStyle
        formatter.maximumUnitCount = 1
        formatter.allowedUnits = (dateComponents.year ?? 0) >= 2 ? .year : .day

        return formatter.string(from: start, to: max(start, end))
    }

    private static func isLessThanADayLeft(dateComponents: DateComponents) -> Bool {
        let year = dateComponents.year ?? 0
        let day = dateComponents.day ?? 0

        return (year == 0) && (day == 0)
    }
}
