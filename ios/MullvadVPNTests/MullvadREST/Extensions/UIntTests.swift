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

@testable import MullvadREST

class UIntTests: XCTestCase {
    func testCountingSets() {
        for setSize in UInt(1)..<20 {
            let sampleSize: UInt = (setSize * 2) - 1

            var count: UInt = 0
            (UInt(0)...sampleSize).forEach { index in
                count = count == setSize ? 1 : count + 1

                let lowerHalfCount = count - 1
                let upperHalfCount = lowerHalfCount + setSize

                XCTAssertEqual(
                    index.isOrdered(nth: count, forEverySetOf: setSize),
                    index == lowerHalfCount || index == upperHalfCount
                )
            }
        }
    }
}
