//
//  IPOverrideTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-01-30.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadSettings
import MullvadTypes
import XCTest

final class IPOverrideTests: XCTestCase {
    let repository = IPOverrideRepository()

    func testCanParseOverrides() throws {
        XCTAssertNoThrow(try parseData(from: overrides))
    }

    func testCanParseOverrideToInternalType() throws {
        let overrides = try parseData(from: overrides)
        overrides.forEach { override in
            if let ipv4Address = override.ipv4Address {
                XCTAssertNotNil(AnyIPAddress(ipv4Address.debugDescription))
            }
            if let ipv6Address = override.ipv6Address {
                XCTAssertNotNil(AnyIPAddress(ipv6Address.debugDescription))
            }
        }
    }

    func testFailedToParseOverridesWithUnsupportedKeys() throws {
        XCTAssertThrowsError(try parseData(from: overridesWithUnsupportedKeys))
    }

    func testFailedToParseOverridesWithMalformedValues() throws {
        XCTAssertThrowsError(try parseData(from: overridesWithMalformedValues))
    }

    func testCreateOverrideWithOneAddress() throws {
        XCTAssertNoThrow(try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: nil))
        XCTAssertNoThrow(try IPOverride(hostname: "Host 1", ipv4Address: nil, ipv6Address: .any))
    }

    func testFailedToCreateOverrideWithNoAddresses() throws {
        XCTAssertThrowsError(try IPOverride(hostname: "Host 1", ipv4Address: nil, ipv6Address: nil))
    }
}

extension IPOverrideTests {
    private func parseData(from overrideString: String) throws -> [IPOverride] {
        let data = overrideString.data(using: .utf8)!
        let overrides = try repository.parse(data: data)

        return overrides
    }
}

extension IPOverrideTests {
    private var overrides: String {
        return """
        {
            "relay_overrides": [
                {
                    "hostname": "Host 1",
                    "ipv4_addr_in": "127.0.0.1",
                    "ipv6_addr_in": "::"
                },
                {
                    "hostname": "Host 2",
                    "ipv4_addr_in": "127.0.0.2",
                    "ipv6_addr_in": "::1"
                }
            ]
        }
        """
    }

    private var overridesWithUnsupportedKeys: String {
        return """
        "{
            "relay_overrides": [{
                "name": "Host 1",
                "hostname": "Host 1",
                "ipv4_addr_in": "127.0.0.1",
                "ipv6_addr_in": "::"
            }]
        }
        """
    }

    private var overridesWithMalformedValues: String {
        return """
        "{
            "relay_overrides": [{
                "hostname": "Host 1",
                "ipv4_addr_in": "127.0.0",
                "ipv6_addr_in": "::"
            }]
        }
        """
    }
}
