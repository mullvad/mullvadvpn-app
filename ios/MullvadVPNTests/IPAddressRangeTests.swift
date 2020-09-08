//
//  IPAddressRangeTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 08/09/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import XCTest

class IPAddressRangeTests: XCTestCase {

    func testParsingValidIPv4AddressRange() throws {
        let addr = try IPAddressRange(string: "127.0.0.1/32")
        XCTAssertEqual("\(addr)", "127.0.0.1/32")
    }

    func testParsingValidIPv6AddressRange() throws {
        let addr = try IPAddressRange(string: "::1/128")
        XCTAssertEqual("\(addr)", "::1/128")
    }

    func testParsingIPv4AddressWithoutNetworkPrefix() throws {
        let addr = try IPAddressRange(string: "127.0.0.1")
        XCTAssertEqual("\(addr)", "127.0.0.1/32")
    }

    func testParsingIPv6AddressWithoutNetworkPrefix() throws {
        let addr = try IPAddressRange(string: "::1")
        XCTAssertEqual("\(addr)", "::1/128")
    }

    func testParsingInvalidIPv4AddressNetworkPrefix() throws {
        let addr = try IPAddressRange(string: "127.0.0.1/33")
        XCTAssertEqual("\(addr)", "127.0.0.1/32")
    }

    func testParsingInvalidIPv6AddressNetworkPrefix() throws {
        let addr = try IPAddressRange(string: "::1/129")
        XCTAssertEqual("\(addr)", "::1/128")
    }

    func testParsingInvalidIPAddress() throws {
        XCTAssertThrowsError(try IPAddressRange(string: "1.2.3.4.5/32")) { (error) in
            XCTAssertEqual(error as? IPAddressRangeParseError, IPAddressRangeParseError.parseAddress("1.2.3.4.5"))
        }
    }

    func testParsingEmptyNetworkPrefix() throws {
        XCTAssertThrowsError(try IPAddressRange(string: "::1/")) { (error) in
            XCTAssertEqual(error as? IPAddressRangeParseError, IPAddressRangeParseError.parsePrefix(""))
        }
    }
}
