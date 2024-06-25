//
//  MigrationManagerTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-17.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

final class MigrationManagerTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()

    var manager: MigrationManager!
    override class func setUp() {
        SettingsManager.unitTestStore = store
    }

    override class func tearDown() {
        SettingsManager.unitTestStore = nil
    }

    override func setUpWithError() throws {
        manager = MigrationManager()
    }

    func testNothingToMigrate() throws {
        let store = Self.store
        let settings = LatestTunnelSettings()
        try SettingsManager.writeSettings(settings)

        let nothingToMigrateExpectation = expectation(description: "No migration")
        manager.migrateSettings(store: store) { result in
            if case .nothing = result {
                nothingToMigrateExpectation.fulfill()
            }
        }
        wait(for: [nothingToMigrateExpectation], timeout: .UnitTest.timeout)
    }

    func testNothingToMigrateWhenSettingsAreNotFound() throws {
        let store = InMemorySettingsStore<KeychainError>()
        SettingsManager.unitTestStore = store

        let nothingToMigrateExpectation = expectation(description: "No migration")
        manager.migrateSettings(store: store) { result in
            if case .nothing = result {
                nothingToMigrateExpectation.fulfill()
            }
        }
        wait(for: [nothingToMigrateExpectation], timeout: .UnitTest.timeout)

        // Reset the `SettingsManager` unit test store to avoid affecting other tests
        // since it's a globally shared instance
        SettingsManager.unitTestStore = Self.store
    }

    func testFailedMigration() throws {
        let store = Self.store
        let failedMigrationExpectation = expectation(description: "Failed migration")
        manager.migrateSettings(store: store) { result in
            if case .failure = result {
                failedMigrationExpectation.fulfill()
            }
        }
        wait(for: [failedMigrationExpectation], timeout: .UnitTest.timeout)
    }

    func testFailedMigrationResetsSettings() throws {
        let store = Self.store
        let data = try XCTUnwrap("Migration test".data(using: .utf8))
        try store.write(data, for: .settings)
        try store.write(data, for: .deviceState)

        // Failed migration should reset settings and device state keys
        manager.migrateSettings(store: store) { _ in }

        let assertDeletionFor: (SettingsKey) throws -> Void = { key in
            try XCTAssertThrowsError(store.read(key: key)) { thrownError in
                XCTAssertTrue(thrownError is SettingNotFound)
            }
        }

        try assertDeletionFor(.deviceState)
        try assertDeletionFor(.lastUsedAccount)
    }

    func testFailedMigrationIfRecordedSettingsVersionHigherThanLatestSettings() throws {
        let store = Self.store
        let settings = FutureVersionSettings()
        try write(settings: settings, version: Int.max - 1, in: store)

        manager.migrateSettings(store: store) { _ in }

        let assertDeletionFor: (SettingsKey) throws -> Void = { key in
            try XCTAssertThrowsError(store.read(key: key)) { thrownError in
                XCTAssertTrue(thrownError is SettingNotFound)
            }
        }

        try assertDeletionFor(.deviceState)
        try assertDeletionFor(.lastUsedAccount)
    }

    func testFailedMigrationCorruptedSchemaResetsSettings() throws {
        let store = Self.store
        let settings = FutureVersionSettings()
        try write(settings: settings, version: -42, in: store)

        let failedMigrationExpectation = expectation(description: "Failed migration")
        manager.migrateSettings(store: store) { result in
            if case .failure = result {
                failedMigrationExpectation.fulfill()
            }
        }
        wait(for: [failedMigrationExpectation], timeout: .UnitTest.timeout)
    }

    func testSuccessfulMigrationFromV4ToLatest() throws {
        var settingsV4 = TunnelSettingsV4()
        let relayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("jp", "osa")]))
        )

        settingsV4.relayConstraints = relayConstraints
        settingsV4.tunnelQuantumResistance = .off
        settingsV4.wireGuardObfuscation = WireGuardObfuscationSettings(state: .off, port: .automatic)

        try migrateToLatest(settingsV4, version: .v4)

        // Once the migration is done, settings should have been updated to the latest available version
        // Verify that the old settings are still valid
        let latestSettings = try SettingsManager.readSettings()
        XCTAssertEqual(settingsV4.relayConstraints, latestSettings.relayConstraints)
        XCTAssertEqual(settingsV4.tunnelQuantumResistance, latestSettings.tunnelQuantumResistance)
        XCTAssertEqual(settingsV4.wireGuardObfuscation, latestSettings.wireGuardObfuscation)
    }

    func testSuccessfulMigrationFromV3ToLatest() throws {
        var settingsV3 = TunnelSettingsV3()
        let relayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("jp", "osa")]))
        )

        settingsV3.relayConstraints = relayConstraints
        settingsV3.dnsSettings = DNSSettings()
        settingsV3.wireGuardObfuscation = WireGuardObfuscationSettings(state: .on, port: .port80)

        try migrateToLatest(settingsV3, version: .v3)

        // Once the migration is done, settings should have been updated to the latest available version
        // Verify that the old settings are still valid
        let latestSettings = try SettingsManager.readSettings()
        XCTAssertEqual(settingsV3.relayConstraints, latestSettings.relayConstraints)
        XCTAssertEqual(settingsV3.wireGuardObfuscation, latestSettings.wireGuardObfuscation)
    }

    func testSuccessfulMigrationFromV2ToLatest() throws {
        var settingsV2 = TunnelSettingsV2()
        let osakaRelayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("jp", "osa")]))
        )

        settingsV2.relayConstraints = osakaRelayConstraints

        try migrateToLatest(settingsV2, version: .v2)

        let latestSettings = try SettingsManager.readSettings()
        XCTAssertEqual(osakaRelayConstraints, latestSettings.relayConstraints)
    }

    func testSuccessfulMigrationFromV1ToLatest() throws {
        var settingsV1 = TunnelSettingsV1()
        let osakaRelayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("jp", "osa")]))
        )

        settingsV1.relayConstraints = osakaRelayConstraints

        try migrateToLatest(settingsV1, version: .v1)

        // Once the migration is done, settings should have been updated to the latest available version
        // Verify that the old settings are still valid
        let latestSettings = try SettingsManager.readSettings()
        XCTAssertEqual(osakaRelayConstraints, latestSettings.relayConstraints)
    }

    private func migrateToLatest(_ settings: any TunnelSettings, version: SchemaVersion) throws {
        let store = Self.store
        try write(settings: settings, version: version.rawValue, in: store)

        let successfulMigrationExpectation = expectation(description: "Successful migration")
        manager.migrateSettings(store: store) { result in
            if case .success = result {
                successfulMigrationExpectation.fulfill()
            }
        }
        wait(for: [successfulMigrationExpectation], timeout: .UnitTest.timeout)
    }

    private func write(settings: any TunnelSettings, version: Int, in store: SettingsStore) throws {
        let parser = SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
        let payload = try parser.producePayload(settings, version: version)
        try store.write(payload, for: .settings)
    }
}

private struct FutureVersionSettings: TunnelSettings {
    func upgradeToNextVersion() -> TunnelSettings { self }
}

struct SettingNotFound: Error, Instantiable {}

extension KeychainError: Instantiable {
    init() {
        self = KeychainError.itemNotFound
    }
}
