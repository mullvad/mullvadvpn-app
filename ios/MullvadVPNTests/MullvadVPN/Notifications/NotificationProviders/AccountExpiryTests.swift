//
//  AccountExpiryTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2023-11-07.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import XCTest

class AccountExpiryTests: XCTestCase {
    private let calendar = Calendar.current

    func testNoDateDuration() {
        let accountExpiry = AccountExpiry()
        XCTAssertNil(accountExpiry.nextTriggerDate(for: .system))
        XCTAssertNil(accountExpiry.nextTriggerDate(for: .inApp))
    }

    func testDateNowDuration() {
        let accountExpiry = AccountExpiry(expiryDate: Date())
        XCTAssertNil(accountExpiry.nextTriggerDate(for: .system))
        XCTAssertNotNil(accountExpiry.nextTriggerDate(for: .inApp)) // In-app expiry triggers on same date as well.
    }

    func testDateInPastDuration() {
        let accountExpiry = AccountExpiry(expiryDate: Date().addingTimeInterval(-10))
        XCTAssertNil(accountExpiry.nextTriggerDate(for: .system))
        XCTAssertNil(accountExpiry.nextTriggerDate(for: .inApp))
    }

    func testDateInFutureDuration() {
        let accountExpiry = AccountExpiry(expiryDate: calendar.date(byAdding: .day, value: 1, to: Date()))

        XCTAssertNotNil(accountExpiry.nextTriggerDate(for: .system))
        XCTAssertNotNil(accountExpiry.nextTriggerDate(for: .inApp))
    }

    func testNumberOfTriggerDates() {
        var accountExpiry = AccountExpiry(
            expiryDate: calendar.date(
                byAdding: .day,
                value: AccountExpiry.Trigger.system.dateIntervals.max()!,
                to: Date()
            )
        )
        XCTAssertEqual(accountExpiry.triggerDates(for: .system).count, AccountExpiry.Trigger.system.dateIntervals.count)

        accountExpiry = AccountExpiry(
            expiryDate: calendar.date(
                byAdding: .day,
                value: AccountExpiry.Trigger.inApp.dateIntervals.max()!,
                to: Date()
            )
        )
        XCTAssertEqual(accountExpiry.triggerDates(for: .inApp).count, AccountExpiry.Trigger.inApp.dateIntervals.count)
    }

    func testDaysRemaining() {
        AccountExpiry.Trigger.system.dateIntervals.forEach { interval in
            let accountExpiry = AccountExpiry(expiryDate: calendar.date(byAdding: .day, value: interval, to: Date()))
            XCTAssertEqual(accountExpiry.daysRemaining(for: .system)?.day, interval)
        }

        AccountExpiry.Trigger.inApp.dateIntervals.forEach { interval in
            let accountExpiry = AccountExpiry(expiryDate: calendar.date(byAdding: .day, value: interval, to: Date()))
            XCTAssertEqual(accountExpiry.daysRemaining(for: .inApp)?.day, interval)
        }
    }
}
