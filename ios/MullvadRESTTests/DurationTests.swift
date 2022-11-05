//
//  DurationTests.swift
//  MullvadRESTTests
//
//  Created by pronebird on 05/11/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import XCTest

final class DurationTests: XCTestCase {
    func testComparable() throws {
        XCTAssertEqual(REST.Duration.milliseconds(1000), .seconds(1))
        XCTAssertEqual(REST.Duration.milliseconds(.max), .seconds(.max))

        XCTAssertGreaterThan(REST.Duration.milliseconds(1001), .seconds(1))
        XCTAssertGreaterThanOrEqual(REST.Duration.seconds(1), .milliseconds(1000))

        XCTAssertLessThan(REST.Duration.milliseconds(999), .seconds(1))
        XCTAssertLessThanOrEqual(REST.Duration.seconds(1), .milliseconds(1000))
    }

    func testMultiplication() throws {
        XCTAssertEqual(REST.Duration.seconds(4) * 4, .seconds(16))
        XCTAssertEqual(REST.Duration.seconds(4) * 4, .seconds(16))
        XCTAssertEqual(REST.Duration.milliseconds(.max - 1) * 2, .milliseconds(.max))
    }

    func testFormat() throws {
        XCTAssertEqual(REST.Duration.milliseconds(999).format(), "999ms")
        XCTAssertEqual(REST.Duration.milliseconds(1000).format(), "1s")
        XCTAssertEqual(REST.Duration.milliseconds(1200).format(), "1.20s")
    }
}
