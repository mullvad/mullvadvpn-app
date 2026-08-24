// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import XCTest

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes

extension MigrationManagerTests {
    func testMigrationDoesNothingIfAnotherProcessIsRunningUpdates() throws {
        let hostProcess = DispatchQueue(label: "net.tests.HostMigration")
        let packetTunnelProcess = DispatchQueue(label: "net.tests.PacketTunnelMigration")
        let osakaRelayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("jp", "osa")]))
        )
        var settingsV1 = TunnelSettingsV1()
        settingsV1.relayConstraints = osakaRelayConstraints

        try write(settings: settingsV1, version: SchemaVersion.v1.rawValue, in: store)

        let backgroundMigrationExpectation = expectation(description: "Migration from packet tunnel")
        let foregroundMigrationExpectation = expectation(description: "Migration from host")
        nonisolated(unsafe) var migrationHappenedInPacketTunnel = false
        nonisolated(unsafe) var migrationHappenedInHost = false

        packetTunnelProcess.async { [unowned self] in
            manager.migrateSettings(store: store) { backgroundMigrationResult in
                if case .success = backgroundMigrationResult {
                    migrationHappenedInPacketTunnel = true
                }
                backgroundMigrationExpectation.fulfill()
            }
        }

        hostProcess.async { [unowned self] in
            manager.migrateSettings(store: store) { foregroundMigrationResult in
                if case .success = foregroundMigrationResult {
                    migrationHappenedInHost = true
                }
                foregroundMigrationExpectation.fulfill()
            }
        }

        wait(for: [backgroundMigrationExpectation, foregroundMigrationExpectation], timeout: .UnitTest.timeout * 4)

        // Migration happens either in one process, or the other.
        // This check guarantees it didn't happen in both simultaneously.
        XCTAssertNotEqual(migrationHappenedInPacketTunnel, migrationHappenedInHost)
        let latestSettings = try SettingsManager(store: store).readSettings()
        XCTAssertEqual(osakaRelayConstraints, latestSettings.relayConstraints)
    }
}
