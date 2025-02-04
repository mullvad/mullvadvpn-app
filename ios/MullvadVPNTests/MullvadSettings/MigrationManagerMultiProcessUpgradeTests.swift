//
//  MigrationManagerMultiProcessUpgradeTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2024-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

extension MigrationManagerTests {
    func testMigrationDoesNothingIfAnotherProcessIsRunningUpdates() throws {
        let hostProcess = DispatchQueue(label: "net.tests.HostMigration")
        let packetTunnelProcess = DispatchQueue(label: "net.tests.PacketTunnelMigration")
        let osakaRelayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("jp", "osa")]))
        )
        var settingsV1 = TunnelSettingsV1()
        settingsV1.relayConstraints = osakaRelayConstraints

        try write(settings: settingsV1, version: SchemaVersion.v1.rawValue, in: Self.store)

        let backgroundMigrationExpectation = expectation(description: "Migration from packet tunnel")
        let foregroundMigrationExpectation = expectation(description: "Migration from host")
        var migrationHappenedInPacketTunnel = false
        var migrationHappenedInHost = false

        packetTunnelProcess.async { [unowned self] in
            manager.migrateSettings(store: MigrationManagerTests.store) { backgroundMigrationResult in
                if case .success = backgroundMigrationResult {
                    migrationHappenedInPacketTunnel = true
                }
                backgroundMigrationExpectation.fulfill()
            }
        }

        hostProcess.async { [unowned self] in
            manager.migrateSettings(store: MigrationManagerTests.store) { foregroundMigrationResult in
                if case .success = foregroundMigrationResult {
                    migrationHappenedInHost = true
                }
                foregroundMigrationExpectation.fulfill()
            }
        }

        wait(for: [backgroundMigrationExpectation, foregroundMigrationExpectation], timeout: .UnitTest.timeout)

        // Migration happens either in one process, or the other.
        // This check guarantees it didn't happen in both simultaneously.
        XCTAssertNotEqual(migrationHappenedInPacketTunnel, migrationHappenedInHost)
        let latestSettings = try SettingsManager.readSettings()
        XCTAssertEqual(osakaRelayConstraints, latestSettings.relayConstraints)
    }
}
