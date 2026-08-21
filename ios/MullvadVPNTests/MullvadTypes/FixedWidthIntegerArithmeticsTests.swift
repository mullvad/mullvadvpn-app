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

class FixedWidthIntegerArithmeticsTests: XCTestCase {
    func testSaturatingMultiplication() {
        XCTAssertEqual(Int16.max.saturatingMultiplication(10), .max)
        XCTAssertEqual(Int16.min.saturatingMultiplication(10), .min)
        XCTAssertEqual(Int16.max.saturatingMultiplication(-10), .min)
        XCTAssertEqual(Int16.min.saturatingMultiplication(-10), .max)
    }

    func testSaturatingAddition() {
        XCTAssertEqual(Int16.max.saturatingAddition(1), .max)
        XCTAssertEqual(Int16.min.saturatingAddition(-1), .min)
    }

    func testSaturatingSubtraction() {
        XCTAssertEqual(Int16.min.saturatingSubtraction(100), .min)
        XCTAssertEqual(Int16.max.saturatingSubtraction(-1), .max)
        XCTAssertEqual(Int16.min.saturatingSubtraction(-1), .min + 1)
        XCTAssertEqual(Int16.max.saturatingSubtraction(1), .max - 1)
    }

    func testSaturatingPow() {
        XCTAssertEqual(Int16(-4).saturatingPow(3), -64)
        XCTAssertEqual(Int16.min.saturatingPow(2), .max)
        XCTAssertEqual(Int16.min.saturatingPow(3), .min)
    }
}
