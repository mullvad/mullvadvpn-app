//
//  ExponentialBackoffTests.swift
//  ExponentialBackoffTests
//
//  Created by pronebird on 05/11/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import XCTest

final class ExponentialBackoffTests: XCTestCase {
    func testExponentialBackoff() {
        var backoff = ExponentialBackoff(initial: .seconds(2), multiplier: 3)

        XCTAssertEqual(backoff.next(), .seconds(2))
        XCTAssertEqual(backoff.next(), .seconds(6))
        XCTAssertEqual(backoff.next(), .seconds(18))
    }

    func testAtMaximumValue() {
        var backoff = ExponentialBackoff(initial: .milliseconds(.max - 1), multiplier: 2)

        XCTAssertEqual(backoff.next(), .milliseconds(.max - 1))
        XCTAssertEqual(backoff.next(), .seconds(.max))
        XCTAssertEqual(backoff.next(), .seconds(.max))
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
        var backoff = ExponentialBackoff(initial: .milliseconds(0), multiplier: 10)

        XCTAssertEqual(backoff.next(), .milliseconds(0))
        XCTAssertEqual(backoff.next(), .milliseconds(0))

        backoff = ExponentialBackoff(initial: .milliseconds(1), multiplier: 0)

        XCTAssertEqual(backoff.next(), .milliseconds(1))
        XCTAssertEqual(backoff.next(), .milliseconds(0))
    }

    func testJitter() {
        let initial = REST.Duration.milliseconds(500)
        var iterator = Jittered(ExponentialBackoff(initial: initial, multiplier: 3))

        XCTAssertGreaterThanOrEqual(iterator.next()!, initial)
    }
}
