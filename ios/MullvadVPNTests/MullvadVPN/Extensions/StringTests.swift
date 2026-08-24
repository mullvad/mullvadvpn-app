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
