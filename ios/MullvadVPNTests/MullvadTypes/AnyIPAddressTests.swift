//
//  AnyIPAddressTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-09-24.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadTypes
import XCTest

final class AnyIPAddressTests: XCTestCase {
    func testAnyIPAddressFromString() {
        XCTAssertNil(AnyIPAddress("000"))
        XCTAssertNil(AnyIPAddress("1"))
        XCTAssertNil(AnyIPAddress("abcde"))
        XCTAssertNil(AnyIPAddress("0.1.2.3.5"))
        XCTAssertNil(AnyIPAddress("2a03:1b20:1:f011:bb09"))

        XCTAssertNotNil(AnyIPAddress("0.0"))
        XCTAssertNotNil(AnyIPAddress("0.1"))
        XCTAssertNotNil(AnyIPAddress("192.168.0.1"))
        XCTAssertNotNil(AnyIPAddress("2a03:1b20:1:f011::bb09"))
        XCTAssertNotNil(AnyIPAddress("FE80:0000:0000:0000:0202:B3FF:FE1E:8329"))
    }
}
