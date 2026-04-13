//
//  IPAddressRangeTests.swift
//  MullvadRustRuntimeTests
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import Network
import XCTest

final class IPAddressRangeTests: XCTestCase {
    // MARK: - IPv4 String Parsing

    func testParseIPv4WithPrefix() {
        let range = IPAddressRange(from: "192.168.1.0/24")
        XCTAssertNotNil(range)
        XCTAssertEqual("\(range!.address)", "192.168.1.0")
        XCTAssertEqual(range!.networkPrefixLength, 24)
    }

    func testParseIPv4WithoutPrefix() {
        let range = IPAddressRange(from: "10.0.0.1")
        XCTAssertNotNil(range)
        XCTAssertEqual("\(range!.address)", "10.0.0.1")
        XCTAssertEqual(range!.networkPrefixLength, 32)
    }

    func testParseIPv4DefaultRoute() {
        let range = IPAddressRange(from: "0.0.0.0/0")
        XCTAssertNotNil(range)
        XCTAssertEqual(range!.networkPrefixLength, 0)
    }

    func testParseIPv4FullMask() {
        let range = IPAddressRange(from: "255.255.255.255/32")
        XCTAssertNotNil(range)
        XCTAssertEqual(range!.networkPrefixLength, 32)
    }

    func testParseIPv4PrefixClampedToMax() {
        let range = IPAddressRange(from: "192.168.1.1/33")
        XCTAssertNotNil(range)
        XCTAssertEqual(range!.networkPrefixLength, 32)
    }

    func testParseInvalidString() {
        XCTAssertNil(IPAddressRange(from: "invalid"))
    }

    func testParseTrailingSlash() {
        XCTAssertNil(IPAddressRange(from: "192.168.1.1/"))
    }

    func testParseEmptyString() {
        XCTAssertNil(IPAddressRange(from: ""))
    }

    // MARK: - IPv6 String Parsing

    func testParseIPv6WithPrefix() {
        let range = IPAddressRange(from: "::ff/64")
        XCTAssertNotNil(range)
        XCTAssertEqual(range!.networkPrefixLength, 64)
    }

    func testParseIPv6WithoutPrefix() {
        let range = IPAddressRange(from: "::1")
        XCTAssertNotNil(range)
        XCTAssertEqual(range!.networkPrefixLength, 128)
    }

    func testParseIPv6DefaultRoute() {
        let range = IPAddressRange(from: "::/0")
        XCTAssertNotNil(range)
        XCTAssertEqual(range!.networkPrefixLength, 0)
    }

    func testParseIPv6FullAddress() {
        let range = IPAddressRange(from: "fe80::1/128")
        XCTAssertNotNil(range)
        XCTAssertEqual(range!.networkPrefixLength, 128)
    }

    func testParseIPv6PrefixClampedToMax() {
        let range = IPAddressRange(from: "::1/129")
        XCTAssertNotNil(range)
        XCTAssertEqual(range!.networkPrefixLength, 128)
    }

    // MARK: - String Representation Roundtrip

    func testStringRepresentationRoundtripIPv4() {
        let original = "192.168.1.0/24"
        let range = IPAddressRange(from: original)!
        XCTAssertEqual(range.description, original)
    }

    func testStringRepresentationRoundtripIPv6() {
        let range = IPAddressRange(from: "::/0")!
        XCTAssertEqual(range.description, "::/0")
    }

    // MARK: - Subnet Mask IPv4

    func testSubnetMaskIPv4Zero() {
        let range = IPAddressRange(from: "0.0.0.0/0")!
        XCTAssertEqual("\(range.subnetMask())", "0.0.0.0")
    }

    func testSubnetMaskIPv4Slash8() {
        let range = IPAddressRange(from: "10.0.0.0/8")!
        XCTAssertEqual("\(range.subnetMask())", "255.0.0.0")
    }

    func testSubnetMaskIPv4Slash24() {
        let range = IPAddressRange(from: "192.168.1.0/24")!
        XCTAssertEqual("\(range.subnetMask())", "255.255.255.0")
    }

    func testSubnetMaskIPv4Slash32() {
        let range = IPAddressRange(from: "192.168.1.1/32")!
        XCTAssertEqual("\(range.subnetMask())", "255.255.255.255")
    }

    // MARK: - Subnet Mask IPv6

    func testSubnetMaskIPv6Zero() {
        let range = IPAddressRange(from: "::/0")!
        let mask = range.subnetMask()
        XCTAssertEqual(mask.rawValue, Data(repeating: 0, count: 16))
    }

    func testSubnetMaskIPv6Slash64() {
        let range = IPAddressRange(from: "::1/64")!
        let mask = range.subnetMask()
        var expected = Data(repeating: 0xff, count: 8)
        expected.append(Data(repeating: 0, count: 8))
        XCTAssertEqual(mask.rawValue, expected)
    }

    func testSubnetMaskIPv6Slash128() {
        let range = IPAddressRange(from: "::1/128")!
        let mask = range.subnetMask()
        XCTAssertEqual(mask.rawValue, Data(repeating: 0xff, count: 16))
    }

    // MARK: - Masked Address

    func testMaskedAddressIPv4() {
        let range = IPAddressRange(from: "192.168.1.100/24")!
        XCTAssertEqual("\(range.maskedAddress())", "192.168.1.0")
    }

    func testMaskedAddressIPv4Slash8() {
        let range = IPAddressRange(from: "10.20.30.40/8")!
        XCTAssertEqual("\(range.maskedAddress())", "10.0.0.0")
    }

    // MARK: - Codable

    func testCodableRoundtrip() throws {
        let range = IPAddressRange(from: "10.64.0.1/32")!
        let encoder = JSONEncoder()
        let data = try encoder.encode(range)
        let decoded = try JSONDecoder().decode(IPAddressRange.self, from: data)
        XCTAssertEqual(range, decoded)
    }

    func testDecodeSingleStringValue() throws {
        let json = "\"10.64.0.1/32\""
        let data = json.data(using: .utf8)!
        let decoded = try JSONDecoder().decode(IPAddressRange.self, from: data)
        XCTAssertEqual("\(decoded.address)", "10.64.0.1")
        XCTAssertEqual(decoded.networkPrefixLength, 32)
    }

    func testDecodeIPv6SingleStringValue() throws {
        let json = "\"::ff/64\""
        let data = json.data(using: .utf8)!
        let decoded = try JSONDecoder().decode(IPAddressRange.self, from: data)
        XCTAssertEqual(decoded.networkPrefixLength, 64)
    }

    func testDecodeInvalidStringThrows() {
        let json = "\"not-an-ip\""
        let data = json.data(using: .utf8)!
        XCTAssertThrowsError(try JSONDecoder().decode(IPAddressRange.self, from: data))
    }

    func testEncodesAsSingleString() throws {
        let range = IPAddressRange(from: "192.168.1.0/24")!
        let data = try JSONEncoder().encode(range)
        let string = String(data: data, encoding: .utf8)!
        XCTAssertEqual(string, "\"192.168.1.0/24\"")
    }

    // MARK: - Equatable / Hashable

    func testEqualRanges() {
        let a = IPAddressRange(from: "10.0.0.1/24")!
        let b = IPAddressRange(from: "10.0.0.1/24")!
        XCTAssertEqual(a, b)
        XCTAssertEqual(a.hashValue, b.hashValue)
    }

    func testUnequalPrefix() {
        let a = IPAddressRange(from: "10.0.0.1/24")!
        let b = IPAddressRange(from: "10.0.0.1/32")!
        XCTAssertNotEqual(a, b)
    }

    func testUnequalAddress() {
        let a = IPAddressRange(from: "10.0.0.1/24")!
        let b = IPAddressRange(from: "10.0.0.2/24")!
        XCTAssertNotEqual(a, b)
    }

    // MARK: - Direct Init

    func testDirectInit() {
        let range = IPAddressRange(address: .ipv4(IPv4Address("192.168.1.1")!), networkPrefixLength: 24)
        XCTAssertEqual("\(range.address)", "192.168.1.1")
        XCTAssertEqual(range.networkPrefixLength, 24)
    }
}
