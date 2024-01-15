//
//  IPOverrideRepositoryTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-01-17.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadSettings
import Network
import XCTest

final class IPOverrideRepositoryTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()
    let repository = IPOverrideRepository()

    override class func setUp() {
        SettingsManager.unitTestStore = store
    }

    override class func tearDown() {
        SettingsManager.unitTestStore = nil
    }

    override func tearDownWithError() throws {
        repository.deleteAll()
    }

    func testCanParseOverrides() throws {
        XCTAssertNoThrow(try parseData(from: overrides))
    }

    func testCannotParseOverridesWithUnsupportedKeys() throws {
        XCTAssertThrowsError(try parseData(from: overridesWithUnsupportedKeys))
    }

    func testCannotParseOverridesWithMalformedValues() throws {
        XCTAssertThrowsError(try parseData(from: overridesWithMalformedValues))
    }

    func testCanCreateOverrideWithOneAddress() throws {
        XCTAssertNoThrow(try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: nil))
        XCTAssertNoThrow(try IPOverride(hostname: "Host 1", ipv4Address: nil, ipv6Address: .any))
    }

    func testCannotCreateOverrideWithNoAddresses() throws {
        XCTAssertThrowsError(try IPOverride(hostname: "Host 1", ipv4Address: nil, ipv6Address: nil))
    }

    func testCanAddOverride() throws {
        let override = try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: nil)
        repository.add([override])

        let storedOverrides = repository.fetchAll()
        XCTAssertTrue(storedOverrides.count == 1)
    }

    func testCanAppendOverrideWithDifferentHostname() throws {
        let override1 = try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: nil)
        let override2 = try IPOverride(hostname: "Host 2", ipv4Address: .any, ipv6Address: nil)
        repository.add([override1, override2])

        let storedOverrides = repository.fetchAll()
        XCTAssertTrue(storedOverrides.count == 2)
    }

    func testCanOverwriteOverrideWithSameHostnameButDifferentAddresses() throws {
        let override1 = try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: nil)
        let override2 = try IPOverride(hostname: "Host 1", ipv4Address: .allHostsGroup, ipv6Address: .broadcast)
        repository.add([override1, override2])

        let storedOverrides = repository.fetchAll()
        XCTAssertTrue(storedOverrides.count == 1)
        XCTAssertTrue(storedOverrides.first?.ipv4Address == .allHostsGroup)
        XCTAssertTrue(storedOverrides.first?.ipv6Address == .broadcast)
    }

    func testCannotOverwriteOverrideWithNilAddress() throws {
        let override1 = try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: .broadcast)
        let override2 = try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: nil)
        repository.add([override1, override2])

        let storedOverrides = repository.fetchAll()
        XCTAssertTrue(storedOverrides.count == 1)
        XCTAssertTrue(storedOverrides.first?.ipv6Address == .broadcast)
    }

    func testCanFetchOverrideByHostname() throws {
        let hostname = "Host 1"
        let override = try IPOverride(hostname: hostname, ipv4Address: .any, ipv6Address: nil)
        repository.add([override])

        let storedOverride = repository.fetchByHostname(hostname)
        XCTAssertTrue(storedOverride?.hostname == hostname)
    }

    func testCanDeleteAllOverrides() throws {
        let override = try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: nil)
        repository.add([override])
        repository.deleteAll()

        let storedOverrides = repository.fetchAll()
        XCTAssertTrue(storedOverrides.isEmpty)
    }
}

extension IPOverrideRepositoryTests {
    private func parseData(from overrideString: String) throws -> [IPOverride] {
        let data = overrideString.data(using: .utf8)!
        let overrides = try repository.parseData(data)

        return overrides
    }
}

extension IPOverrideRepositoryTests {
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
