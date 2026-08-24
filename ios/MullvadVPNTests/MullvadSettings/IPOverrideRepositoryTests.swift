// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Network
import XCTest

@testable import MullvadSettings

final class IPOverrideRepositoryTests: XCTestCase {
    let store = InMemorySettingsStore<SettingNotFound>()
    lazy var repository = IPOverrideRepository(settingsStore: store)

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
