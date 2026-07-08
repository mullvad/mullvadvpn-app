//
//  MigrationManagerTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-17.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import XCTest

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes

final class MigrationManagerTests: XCTestCase, @unchecked Sendable {
    let store = InMemorySettingsStore<SettingNotFound>()
    lazy var settingsManager = SettingsManager(store: store)

    var manager: MigrationManager!
    var testFileURL: URL!

    override func setUpWithError() throws {
        testFileURL = FileManager.default.temporaryDirectory
            .appendingPathComponent("MigrationManagerTests-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: testFileURL, withIntermediateDirectories: true)
        manager = MigrationManager(cacheDirectory: testFileURL, settingsManager: settingsManager)
    }

    override func tearDownWithError() throws {
        try FileManager.default.removeItem(at: testFileURL)
    }

    func testNothingToMigrate() throws {
        let settings = LatestTunnelSettings()
        try settingsManager.writeSettings(settings)

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
        settingsManager = SettingsManager(store: store)

        let nothingToMigrateExpectation = expectation(description: "No migration")
        manager.migrateSettings(store: store) { result in
            if case .nothing = result {
                nothingToMigrateExpectation.fulfill()
            }
        }
        wait(for: [nothingToMigrateExpectation], timeout: .UnitTest.timeout)
    }

    func testFailedMigration() throws {
        let failedMigrationExpectation = expectation(description: "Failed migration")
        manager.migrateSettings(store: store) { result in
            if case .failure = result {
                failedMigrationExpectation.fulfill()
            }
        }
        wait(for: [failedMigrationExpectation], timeout: .UnitTest.timeout)
    }

    func testFailedMigrationResetsSettings() throws {
        let data = Data("Migration test".utf8)
        try store.write(data, for: .settings)
        try store.write(data, for: .deviceState)

        // Failed migration should reset settings and device state keys
        manager.migrateSettings(store: store) { _ in }

        let assertDeletionFor: (SettingsKey) throws -> Void = { [store] key in
            try XCTAssertThrowsError(store.read(key: key)) { thrownError in
                XCTAssertTrue(thrownError is SettingNotFound)
            }
        }

        try assertDeletionFor(.deviceState)
        try assertDeletionFor(.lastUsedAccount)
    }

    func testFailedMigrationIfRecordedSettingsVersionHigherThanLatestSettings() throws {
        let settings = FutureVersionSettings()
        try write(settings: settings, version: Int.max - 1, in: settingsManager.store)

        manager.migrateSettings(store: store) { _ in }

        let assertDeletionFor: (SettingsKey) throws -> Void = { [store] key in
            try XCTAssertThrowsError(store.read(key: key)) { thrownError in
                XCTAssertTrue(thrownError is SettingNotFound)
            }
        }

        try assertDeletionFor(.deviceState)
        try assertDeletionFor(.lastUsedAccount)
    }

    func testFailedMigrationCorruptedSchemaResetsSettings() throws {
        let settings = FutureVersionSettings()
        try write(settings: settings, version: -42, in: settingsManager.store)

        let failedMigrationExpectation = expectation(description: "Failed migration")
        manager.migrateSettings(store: store) { result in
            if case .failure = result {
                failedMigrationExpectation.fulfill()
            }
        }
        wait(for: [failedMigrationExpectation], timeout: .UnitTest.timeout)
    }

    func testSuccessfulMigrationFromV7ToLatest() throws {
        var settingsV7 = TunnelSettingsV7()
        let relayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("jp", "osa")]))
        )

        settingsV7.relayConstraints = relayConstraints
        settingsV7.tunnelQuantumResistance = .off
        settingsV7.wireGuardObfuscation = WireGuardObfuscationSettings(
            state: .off,
            udpOverTcpPort: .automatic
        )
        settingsV7.tunnelMultihopState = .off
        settingsV7.daita = .init(daitaState: .on)

        try migrateToLatest(settingsV7, version: .v6)

        // Once the migration is done, settings should have been updated to the latest available version
        // Verify that the old settings are still valid
        let latestSettings = try settingsManager.readSettings()
        XCTAssertEqual(settingsV7.relayConstraints, latestSettings.relayConstraints)
        XCTAssertEqual(settingsV7.tunnelQuantumResistance, latestSettings.tunnelQuantumResistance)
        XCTAssertEqual(settingsV7.wireGuardObfuscation, latestSettings.wireGuardObfuscation)
        XCTAssertEqual(latestSettings.tunnelMultihopState, .never)
        XCTAssertEqual(settingsV7.daita, latestSettings.daita)
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
        let latestSettings = try settingsManager.readSettings()
        XCTAssertEqual(settingsV6.relayConstraints, latestSettings.relayConstraints)
        XCTAssertEqual(settingsV6.tunnelQuantumResistance, latestSettings.tunnelQuantumResistance)
        XCTAssertEqual(settingsV6.wireGuardObfuscation, latestSettings.wireGuardObfuscation)
        XCTAssertEqual(latestSettings.tunnelMultihopState, .never)
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
        let latestSettings = try settingsManager.readSettings()
        XCTAssertEqual(settingsV5.relayConstraints, latestSettings.relayConstraints)
        XCTAssertEqual(settingsV5.tunnelQuantumResistance, latestSettings.tunnelQuantumResistance)
        XCTAssertEqual(settingsV5.wireGuardObfuscation, latestSettings.wireGuardObfuscation)
        XCTAssertEqual(latestSettings.tunnelMultihopState, .never)
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
        let latestSettings = try settingsManager.readSettings()
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
        let latestSettings = try settingsManager.readSettings()
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

        let latestSettings = try settingsManager.readSettings()
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
        let latestSettings = try settingsManager.readSettings()
        XCTAssertEqual(osakaRelayConstraints, latestSettings.relayConstraints)
    }

    /// Settings serialized by ios/2026.1-build5 had `includeAllNetworks` and `localNetworkSharing`
    /// as Bool fields in TunnelSettingsV7. The IAN activation flow changed `includeAllNetworks`
    /// to an `IncludeAllNetworksSettings` struct (absorbing `localNetworkSharing`) without bumping
    /// the schema version, so existing V7 data must still deserialize correctly.
    func testDeserializationOfV7SettingsFromBuild5() throws {
        // Verbatim representation of default V7 settings as written by ios/2026.1-build5.
        let oldSettingsJSON = Data(
            """
            {
                "version": 7,
                "data": {
                    "relayConstraints": {
                        "location": {"only": ["se"]},
                        "locations": {"only": {"locations": [["se"]]}},
                        "entryLocations": {"only": {"locations": [["se"]]}},
                        "exitLocations": {"only": {"locations": [["se"]]}},
                        "port": "any",
                        "filter": "any"
                    },
                    "dnsSettings": {
                        "blockingOptions": 0,
                        "enableCustomDNS": false,
                        "customDNSDomains": []
                    },
                    "wireGuardObfuscation": {
                        "port": 0,
                        "state": {"automatic": {}},
                        "udpOverTcpPort": {"automatic": {}},
                        "shadowsocksPort": {"automatic": {}}
                    },
                    "tunnelQuantumResistance": {"automatic": {}},
                    "tunnelMultihopState": {"off": {}},
                    "daita": {
                        "state": {"off": {}},
                        "daitaState": {"off": {}},
                        "directOnlyState": {"off": {}}
                    },
                    "localNetworkSharing": true,
                    "includeAllNetworks": true
                }
            }
            """.utf8)

        let parser = SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
        let settings = try parser.parsePayload(as: TunnelSettingsV7.self, from: oldSettingsJSON)

        XCTAssertFalse(settings.includeAllNetworks.includeAllNetworksIsEnabled)
        XCTAssertFalse(settings.includeAllNetworks.localNetworkSharingIsEnabled)
    }

    /// Migration test: ensures that previously stored settings using the removed
    /// `automatic` case for `tunnelQuantumResistance` are safely mapped to `.on`.
    /// Prevents crashes and guarantees consistent behavior for existing users
    /// after upgrading to versions where `automatic` no longer exists.
    func testTunnelQuantumResistanceMigratesAutomaticToOn() throws {
        let oldSettingsJSON = Data(
            """
            {
                "version": 4,
                "data": {
                    "relayConstraints": {
                        "location": {"only": ["se"]},
                        "locations": {"only": {"locations": [["se"]]}},
                        "entryLocations": {"only": {"locations": [["se"]]}},
                        "exitLocations": {"only": {"locations": [["se"]]}},
                        "port": "any",
                        "filter": "any"
                    },
                    "dnsSettings": {
                        "blockingOptions": 0,
                        "enableCustomDNS": false,
                        "customDNSDomains": []
                    },
                    "wireGuardObfuscation": {
                        "port": 0,
                        "state": {"automatic": {}},
                        "udpOverTcpPort": {"automatic": {}},
                        "shadowsocksPort": {"automatic": {}}
                    },
                    "tunnelQuantumResistance": {"automatic": {}}
                }
            }
            """.utf8)

        let parser = SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
        let tunnelSettingsV4 = try parser.parsePayload(as: TunnelSettingsV4.self, from: oldSettingsJSON)
        try write(settings: tunnelSettingsV4, version: 4, in: store)

        let successfulMigrationExpectation = expectation(description: "Successful migration")
        manager.migrateSettings(store: store) { result in
            if case .success = result {
                successfulMigrationExpectation.fulfill()
            }
        }
        wait(for: [successfulMigrationExpectation], timeout: .UnitTest.timeout)

        let latestSettingsData = try XCTUnwrap(settingsManager.store.read(key: .settings))
        let latestSettings = try parser.parsePayload(as: LatestTunnelSettings.self, from: latestSettingsData)

        XCTAssertEqual(latestSettings.tunnelQuantumResistance, .on)
    }

    // MARK: - In-memory schema upgrade

    /// The read path upgrades the settings schema in place
    func testReadUpgradingSchemaInMemoryUpgradesOldSchema() throws {
        var settingsV7 = TunnelSettingsV7()
        let relayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("jp", "osa")]))
        )
        settingsV7.relayConstraints = relayConstraints
        settingsV7.tunnelQuantumResistance = .off
        settingsV7.tunnelMultihopState = .off
        settingsV7.daita = .init(daitaState: .on)

        try write(settings: settingsV7, version: SchemaVersion.v7.rawValue, in: store)

        let upgraded = try settingsManager.readSettingsUpgradingSchemaInMemory()

        XCTAssertEqual(upgraded.relayConstraints, settingsV7.relayConstraints)
        XCTAssertEqual(upgraded.tunnelQuantumResistance, settingsV7.tunnelQuantumResistance)
        XCTAssertEqual(upgraded.tunnelMultihopState, .never)
        XCTAssertEqual(upgraded.daita, settingsV7.daita)
    }

    func testReadUpgradingSchemaInMemoryIsIdenticalToRead() throws {
        var settingsV7 = TunnelSettingsV7()
        let relayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("jp", "osa")]))
        )
        settingsV7.relayConstraints = relayConstraints
        settingsV7.tunnelQuantumResistance = .off
        settingsV7.tunnelMultihopState = .off
        settingsV7.daita = .init(daitaState: .on)
        try write(settings: settingsV7, version: SchemaVersion.v7.rawValue, in: store)

        let ephemeral = try settingsManager.readSettingsUpgradingSchemaInMemory()
        manager
            .migrateSettings(store: store) { _ in return }
        let persisted = try settingsManager.readSettings()

        XCTAssertEqual(ephemeral, persisted)
    }

    /// The stored settings must remain at its original schema version.
    func testReadUpgradingSchemaInMemoryDoesNotPersist() throws {
        var settingsV7 = TunnelSettingsV7()
        settingsV7.tunnelMultihopState = .off
        try write(settings: settingsV7, version: SchemaVersion.v7.rawValue, in: store)

        _ = try settingsManager.readSettingsUpgradingSchemaInMemory()

        let parser = SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
        let storedData = try store.read(key: .settings)
        let storedVersion = try parser.parseVersion(data: storedData)
        XCTAssertEqual(storedVersion, SchemaVersion.v7.rawValue)
    }

    /// Current-schema settings are unchanged
    func testReadUpgradingSchemaInMemoryReturnsCurrentSchemaSettings() throws {
        var settings = LatestTunnelSettings()
        settings.relayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("jp", "osa")]))
        )
        try settingsManager.writeSettings(settings)

        let read = try settingsManager.readSettingsUpgradingSchemaInMemory()

        XCTAssertEqual(read, settings)
    }

    /// An unknown (e.g. downgraded / future) schema version cannot be upgraded and fails to be read
    func testReadUpgradingSchemaInMemoryThrowsOnUnsupportedVersion() throws {
        try write(settings: FutureVersionSettings(), version: Int.max - 1, in: store)

        XCTAssertThrowsError(try settingsManager.readSettingsUpgradingSchemaInMemory()) { error in
            XCTAssertTrue(error is UnsupportedSettingsVersionError)
        }
    }

    private func migrateToLatest(_ settings: any TunnelSettings, version: SchemaVersion) throws {
        try write(settings: settings, version: version.rawValue, in: settingsManager.store)

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

    var debugDescription: String {
        "FutureVersionSettings"
    }
}

struct SettingNotFound: Error, Instantiable {}

extension KeychainError: Instantiable {
    init() {
        self = KeychainError.itemNotFound
    }
}
