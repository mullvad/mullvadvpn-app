//
//  IPOverrideRepositoryTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-01-17.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadSettings
import Network
import XCTest

final class IPOverrideRepositoryTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()
    let repository = IPOverrideRepository()

    override static func setUp() {
        SettingsManager.unitTestStore = store
    }

    override static func tearDown() {
        store.reset()
    }

    override func tearDownWithError() throws {
        repository.deleteAll()
    }

    func testAddOverride() throws {
        let override = try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: nil)
        repository.add([override])

        let storedOverrides = repository.fetchAll()
        XCTAssertTrue(storedOverrides.count == 1)
    }

    func testAppendOverrideWithDifferentHostname() throws {
        let override1 = try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: nil)
        repository.add([override1])
        let override2 = try IPOverride(hostname: "Host 2", ipv4Address: .any, ipv6Address: nil)
        repository.add([override2])

        let storedOverrides = repository.fetchAll()
        XCTAssertTrue(storedOverrides.count == 2)
    }

    func testOverwriteOverrideWithSameHostnameButDifferentAddresses() throws {
        let override1 = try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: nil)
        repository.add([override1])
        let override2 = try IPOverride(hostname: "Host 1", ipv4Address: .allHostsGroup, ipv6Address: .broadcast)
        repository.add([override2])

        let storedOverrides = repository.fetchAll()
        XCTAssertTrue(storedOverrides.count == 1)
        XCTAssertTrue(storedOverrides.first?.ipv4Address == .allHostsGroup)
        XCTAssertTrue(storedOverrides.first?.ipv6Address == .broadcast)
    }

    func testFailedToOverwriteOverrideWithNilAddress() throws {
        let override1 = try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: .broadcast)
        repository.add([override1])
        let override2 = try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: nil)
        repository.add([override2])

        let storedOverrides = repository.fetchAll()
        XCTAssertTrue(storedOverrides.count == 1)
        XCTAssertTrue(storedOverrides.first?.ipv6Address == .broadcast)
    }

    func testDeleteAllOverrides() throws {
        let override = try IPOverride(hostname: "Host 1", ipv4Address: .any, ipv6Address: nil)
        repository.add([override])
        repository.deleteAll()

        let storedOverrides = repository.fetchAll()
        XCTAssertTrue(storedOverrides.isEmpty)
    }
}
