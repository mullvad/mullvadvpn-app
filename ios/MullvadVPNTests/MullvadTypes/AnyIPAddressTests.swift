//
//  AnyIPAddressTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-09-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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
        XCTAssertNotNil(AnyIPAddress("2001:db8::42:0:8a2e:370:7334"))
        XCTAssertNotNil(AnyIPAddress("::1"))
        XCTAssertNotNil(AnyIPAddress("::"))
        XCTAssertNotNil(AnyIPAddress("::ffff:192.168.1.1"))
        XCTAssertNotNil(AnyIPAddress("fe80::1ff:fe23:4567:890a"))
        XCTAssertNotNil(AnyIPAddress("ff02::1"))
        XCTAssertNotNil(AnyIPAddress("2001:0db8:85a3::8a2e:0370:7334"))
    }
}
