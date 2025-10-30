//
//  ExponentialBackoffTests.swift
//  ExponentialBackoffTests
//
//  Created by pronebird on 05/11/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import XCTest

@testable import MullvadREST

final class ExponentialBackoffTests: XCTestCase {
    func testExponentialBackoff() {
        var backoff = ExponentialBackoff(initial: .seconds(2), multiplier: 3, maxDelay: .seconds(18))

        XCTAssertEqual(backoff.next(), .seconds(2))
        XCTAssertEqual(backoff.next(), .seconds(6))
        XCTAssertEqual(backoff.next(), .seconds(18))
    }

    func testAtMaximumValue() {
        var backoff = ExponentialBackoff(initial: .milliseconds(.max - 1), multiplier: 2, maxDelay: .seconds(.max - 1))

        XCTAssertEqual(backoff.next(), .milliseconds(.max - 1))
    }

    func testMaximumBound() {
        var backoff = ExponentialBackoff(
            initial: .milliseconds(2),
            multiplier: 3,
            maxDelay: .milliseconds(7)
        )

        XCTAssertEqual(backoff.next(), .milliseconds(2))
        XCTAssertEqual(backoff.next(), .milliseconds(6))
        XCTAssertEqual(backoff.next(), .milliseconds(7))
    }

    func testMinimumValue() {
        var backoff = ExponentialBackoff(initial: .milliseconds(0), multiplier: 10, maxDelay: .milliseconds(0))

        XCTAssertEqual(backoff.next(), .milliseconds(0))
        XCTAssertEqual(backoff.next(), .milliseconds(0))

        backoff = ExponentialBackoff(initial: .milliseconds(1), multiplier: 0, maxDelay: .zero)

        XCTAssertEqual(backoff.next(), .milliseconds(0))
        XCTAssertEqual(backoff.next(), .milliseconds(0))
    }

    func testJitter() {
        let initial: Duration = .milliseconds(500)
        var iterator = Jittered(ExponentialBackoff(initial: initial, multiplier: 3, maxDelay: .milliseconds(1500)))

        XCTAssertGreaterThanOrEqual(iterator.next()!, initial)
    }
}
