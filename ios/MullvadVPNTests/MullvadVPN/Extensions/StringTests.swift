//
//  StringTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 27/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import XCTest

class StringTests: XCTestCase {
    func testEmptyString() {
        XCTAssertTrue("".split(every: 4).isEmpty)
    }

    func testString() {
        XCTAssertEqual("12345678".split(every: 4), ["1234", "5678"])
    }

    func testOddString() {
        XCTAssertEqual("123456789".split(every: 4), ["1234", "5678", "9"])
    }

    func testStringShorterThanLength() {
        XCTAssertEqual("1".split(every: 4), ["1"])
    }
}
