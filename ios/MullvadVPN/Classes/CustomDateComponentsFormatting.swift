// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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

        guard !isLessThanOneDay(dateComponents: dateComponents) else {
            return NSLocalizedString("Less than a day", comment: "")
        }

        let formatter = DateComponentsFormatter()
        formatter.calendar = calendar
        formatter.unitsStyle = unitsStyle
        formatter.maximumUnitCount = 1
        formatter.allowedUnits = (dateComponents.year ?? 0) >= 2 ? .year : .day

        return formatter.string(from: start, to: max(start, end))
    }

    private static func isLessThanOneDay(dateComponents: DateComponents) -> Bool {
        let year = dateComponents.year ?? 0
        let day = dateComponents.day ?? 0

        return (year == 0) && (day == 0)
    }
}
