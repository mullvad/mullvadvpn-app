// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import XCTest

class CustomDateComponentsFormattingTests: XCTestCase {
    func testEqualToTwoYearsFormatting() throws {
        var dateComponents = DateComponents()
        dateComponents.year = 2

        let (startDate, endDate) = makeDateRange(addingComponents: dateComponents)

        let result = CustomDateComponentsFormatting.localizedString(
            from: startDate,
            to: endDate,
            calendar: calendar,
            unitsStyle: .full
        )

        XCTAssertEqual(result, "2 years")
    }

    func testTimeIsPassedFormatting() throws {
        var dateComponents = DateComponents()
        dateComponents.day = -1

        let (startDate, endDate) = makeDateRange(addingComponents: dateComponents)

        let result = CustomDateComponentsFormatting.localizedString(
            from: startDate,
            to: endDate,
            calendar: calendar,
            unitsStyle: .full
        )

        XCTAssertEqual(result, "Less than a day")
    }

    func testLessThanTwoYearsFormatting() throws {
        var dateComponents = DateComponents()
        dateComponents.day = 365

        let (startDate, endDate) = makeDateRange(addingComponents: dateComponents)

        let result = CustomDateComponentsFormatting.localizedString(
            from: startDate,
            to: endDate,
            calendar: calendar,
            unitsStyle: .full
        )

        XCTAssertEqual(result, "365 days")
    }

    func testCloseToOneDayFormatting() throws {
        var dateComponents = DateComponents()
        dateComponents.hour = 23
        dateComponents.minute = 30

        let (startDate, endDate) = makeDateRange(addingComponents: dateComponents)

        let result = CustomDateComponentsFormatting.localizedString(
            from: startDate,
            to: endDate,
            calendar: calendar,
            unitsStyle: .full
        )

        XCTAssertEqual(result, "Less than a day")
    }

    private func makeDateRange(addingComponents dateComponents: DateComponents) -> (Date, Date) {
        let startDate = Date()
        let endDate = calendar.date(byAdding: dateComponents, to: startDate)!

        return (startDate, endDate)
    }

    private var calendar: Calendar {
        var calendar = Calendar(identifier: .gregorian)
        calendar.locale = Locale(identifier: "en_US_POSIX")
        return calendar
    }
}
