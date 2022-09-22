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
    /// 1. Intervals between 23h 30m - 23h 59m are rounded to 1 day to fix the iOS SDK bug which
    ///    results in the wrong output ("0 months").
    /// 2. Intervals between 26 and 90 days are formatted in days quantity.
    /// 3. Produce "Less than a minute" message for intervals below 1 minute.
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
            .dateComponents([.day, .hour, .minute, .second], from: start, to: end)

        let days = dateComponents.day ?? 0
        let hours = dateComponents.hour ?? 0
        let minutes = dateComponents.minute ?? 0
        let seconds = dateComponents.second ?? 0

        if days == 0, hours == 0, minutes == 0, seconds < 60 {
            return NSLocalizedString(
                "LESS_THAN_ONE_MINUTE",
                tableName: "CustomDateComponentsFormatting",
                value: "Less than a minute",
                comment: "Phrase used for less than 1 minute duration."
            )
        } else if days == 0, hours == 23, minutes >= 30 {
            return formatter.string(from: DateComponents(calendar: calendar, day: 1))
        } else if days >= 1, days <= 90 {
            formatter.allowedUnits = [.day]
            return formatter.string(from: dateComponents)
        } else {
            return formatter.string(from: start, to: end)
        }
    }
}
