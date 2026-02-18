//
//  TunnelControlPageTests.swift
//  MullvadVPNUITests
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import XCTest

class TunnelControlPageTests: XCTestCase {
    func testConnectionDetailValueStripsInPrefix() {
        let result = TunnelControlPage.connectionDetailValue(from: "In 85.203.53.104:56678 UDP")
        XCTAssertEqual(result, "85.203.53.104:56678 UDP")
    }

    func testConnectionDetailValueStripsOutIPv4Prefix() {
        let result = TunnelControlPage.connectionDetailValue(from: "Out IPv4 192.168.1.1")
        XCTAssertEqual(result, "192.168.1.1")
    }

    func testConnectionDetailValueStripsOutIPv6Prefix() {
        let result = TunnelControlPage.connectionDetailValue(from: "Out IPv6 2001:db8::1")
        XCTAssertEqual(result, "2001:db8::1")
    }

    func testConnectionDetailValueReturnsLabelWhenNoAddressFound() {
        let result = TunnelControlPage.connectionDetailValue(from: "No address")
        XCTAssertEqual(result, "No address")
    }

    func testConnectionDetailValueHandlesIPAddressOnly() {
        let result = TunnelControlPage.connectionDetailValue(from: "In 85.203.53.104")
        XCTAssertEqual(result, "85.203.53.104")
    }
}
