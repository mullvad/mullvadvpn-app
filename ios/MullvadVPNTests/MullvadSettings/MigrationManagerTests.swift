//
//  MigrationManagerTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-17.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

final class MigrationManagerTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()

    var manager: MigrationManager!
    var testFileURL: URL!
    override static func setUp() {
        SettingsManager.unitTestStore = store
    }

    override static func tearDown() {
        SettingsManager.unitTestStore = nil
    }

    override func setUpWithError() throws {
        testFileURL = FileManager.default.temporaryDirectory
            .appendingPathComponent("MigrationManagerTests-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: testFileURL, withIntermediateDirectories: true)
        manager = MigrationManager(cacheDirectory: testFileURL)
    }

    override func tearDownWithError() throws {
        try FileManager.default.removeItem(at: testFileURL)
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
        let data = Data("Migration test".utf8)
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

    func testSuccessfulMigrationFromV6ToLatest() throws {
        var settingsV6 = TunnelSettingsV6()
        let relayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("jp", "osa")]))
        )

        settingsV6.relayConstraints = relayConstraints
        settingsV6.tunnelQuantumResistance = .off
        settingsV6.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .off,
            udpOverTcpPort: .automatic
        )
        settingsV6.tunnelMultihopState = .off
        settingsV6.daita = .init(daitaState: .on)

        try migrateToLatest(settingsV6, version: .v6)

        // Once the migration is done, settings should have been updated to the latest available version
        // Verify that the old settings are still valid
        let latestSettings = try SettingsManager.readSettings()
        XCTAssertEqual(settingsV6.relayConstraints, latestSettings.relayConstraints)
        XCTAssertEqual(settingsV6.tunnelQuantumResistance, latestSettings.tunnelQuantumResistance)
        XCTAssertEqual(settingsV6.wireGuardObfuscation, latestSettings.wireGuardObfuscation)
        XCTAssertEqual(settingsV6.tunnelMultihopState, latestSettings.tunnelMultihopState)
        XCTAssertEqual(settingsV6.daita, latestSettings.daita)
    }

    func testSuccessfulMigrationFromV5ToLatest() throws {
        var settingsV5 = TunnelSettingsV5()
        let relayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("jp", "osa")]))
        )

        settingsV5.relayConstraints = relayConstraints
        settingsV5.tunnelQuantumResistance = .off
        settingsV5.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .off,
            udpOverTcpPort: .automatic
        )
        settingsV5.tunnelMultihopState = .off

        try migrateToLatest(settingsV5, version: .v5)

        // Once the migration is done, settings should have been updated to the latest available version
        // Verify that the old settings are still valid
        let latestSettings = try SettingsManager.readSettings()
        XCTAssertEqual(settingsV5.relayConstraints, latestSettings.relayConstraints)
        XCTAssertEqual(settingsV5.tunnelQuantumResistance, latestSettings.tunnelQuantumResistance)
        XCTAssertEqual(settingsV5.wireGuardObfuscation, latestSettings.wireGuardObfuscation)
        XCTAssertEqual(settingsV5.tunnelMultihopState, latestSettings.tunnelMultihopState)
    }

    func testSuccessfulMigrationFromV4ToLatest() throws {
        var settingsV4 = TunnelSettingsV4()
        let relayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("jp", "osa")]))
        )

        settingsV4.relayConstraints = relayConstraints
        settingsV4.tunnelQuantumResistance = .off
        settingsV4.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .off,
            udpOverTcpPort: .automatic
        )

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
        settingsV3.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .udpOverTcp,
            udpOverTcpPort: .port80
        )

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

    func write(settings: any TunnelSettings, version: Int, in store: SettingsStore) throws {
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
