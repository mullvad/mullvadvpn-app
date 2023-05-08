//
//  CustomDateComponentsFormattingTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 14/05/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

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

    func testMoreThanTwoYearsFormatting() throws {
        var dateComponents = DateComponents()
        dateComponents.year = 3

        let (startDate, endDate) = makeDateRange(addingComponents: dateComponents)

        let result = CustomDateComponentsFormatting.localizedString(
            from: startDate,
            to: endDate,
            calendar: calendar,
            unitsStyle: .full
        )

        XCTAssertEqual(result, "3 years")
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

        XCTAssertEqual(result, "0 days")
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
