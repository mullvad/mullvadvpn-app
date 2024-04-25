//
//  DurationTests.swift
//  MullvadRESTTests
//
//  Created by pronebird on 05/11/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import MullvadTypes
import XCTest

final class DurationTests: XCTestCase {
    func testComparable() throws {
        XCTAssertEqual(Duration.milliseconds(1000), .seconds(1))
        XCTAssertEqual(Duration.seconds(60), .minutes(1))
        XCTAssertEqual(Duration.minutes(60), .hours(1))
        XCTAssertEqual(Duration.hours(24), .days(1))

        XCTAssertTrue(Duration.days(1) == 86400.0)
        XCTAssertTrue(Duration.milliseconds(1234) == 1.234)

        XCTAssertTrue(Duration.milliseconds(.max) == 9.223372036854775e+15)
        XCTAssertEqual(Duration.milliseconds(.max).milliseconds, 9223372036854775807)
        XCTAssertEqual(Duration.seconds(.max).milliseconds, 9223372036854775807)

        XCTAssertLessThan(Duration.milliseconds(999), .seconds(1))
        XCTAssertGreaterThan(Duration.milliseconds(1001), .seconds(1))
        XCTAssertTrue(1.1 > Duration.milliseconds(1001))
        XCTAssertTrue(1.0 < Duration.milliseconds(1001))
    }

    func testAddition() throws {
        XCTAssertEqual(Duration.seconds(4) + .seconds(116), .minutes(2))
        XCTAssertEqual(Duration.minutes(4) + .seconds(.max), .milliseconds(.max))
    }

    func testSubtraction() throws {
        XCTAssertEqual(Duration.minutes(4) - .minutes(1), .seconds(180))
        XCTAssertEqual(Duration.seconds(4) - .seconds(64), .minutes(-1))
    }

    func testMultiplication() throws {
        XCTAssertEqual(Duration.seconds(4) * 2.0, .seconds(8))
        XCTAssertEqual(Duration.seconds(4) * 4, .seconds(16))
        XCTAssertEqual(Duration.milliseconds(20000) * 3, .minutes(1))
        XCTAssertEqual(Duration.milliseconds(.max - 1) * 2, .milliseconds(.max))
        XCTAssertEqual(
            Duration.milliseconds(.max) * (Double(Int.max) + 1.0),
            .milliseconds(Int(Double(Int.max).nextDown))
        )
    }

    func testDivision() throws {
        XCTAssertEqual(Duration.seconds(4) / 4, .seconds(1))
        XCTAssertEqual(Duration.milliseconds(21000) / 3, .seconds(7))
        XCTAssertEqual(Duration.minutes(3) / 3, .seconds(60))
    }

    func testLogFormat() throws {
        XCTAssertEqual(Duration.milliseconds(999).logFormat(), "999ms")
        XCTAssertEqual(Duration.milliseconds(1000).logFormat(), "1s")
        XCTAssertEqual(Duration.milliseconds(1200).logFormat(), "1.20s")
    }
}

private extension Duration {
    static func == (lhs: Duration, rhs: Double) -> Bool {
        return lhs.timeInterval == rhs
    }
}
