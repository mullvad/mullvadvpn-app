//
//  AccountExpiryTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2023-11-07.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import XCTest

class AccountExpiryTests: XCTestCase {
    func testNoDateReturnsNoDuration() {
        let accountExpiry = AccountExpiry()
        XCTAssertNil(accountExpiry.formattedDuration)
    }

    func testDateNowReturnsNoDuration() {
        let accountExpiry = AccountExpiry(expiryDate: Date())
        XCTAssertNil(accountExpiry.formattedDuration)
    }

    func testDateInPastReturnsNoDuration() {
        let accountExpiry = AccountExpiry(expiryDate: Date().addingTimeInterval(-10))
        XCTAssertNil(accountExpiry.formattedDuration)
    }

    func testDateWithinTriggerIntervalReturnsDuration() {
        let date = Calendar.current.date(
            byAdding: .day,
            value: NotificationConfiguration.closeToExpiryTriggerInterval - 1,
            to: Date()
        )

        let accountExpiry = AccountExpiry(expiryDate: date)
        XCTAssertNotNil(accountExpiry.formattedDuration)
    }

    func testDateNotWithinTriggerIntervalReturnsNoDuration() {
        let date = Calendar.current.date(
            byAdding: .day,
            value: NotificationConfiguration.closeToExpiryTriggerInterval + 1,
            to: Date()
        )

        let accountExpiry = AccountExpiry(expiryDate: date)
        XCTAssertNil(accountExpiry.formattedDuration)
    }

}
