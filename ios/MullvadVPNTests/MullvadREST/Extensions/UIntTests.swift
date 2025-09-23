//
//  UIntTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
