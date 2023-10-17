//
//  MigrationManagerTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-17.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

final class MigrationManagerTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()

    var manager: MigrationManager!
    var proxyFactory: REST.ProxyFactory!
    override class func setUp() {
        SettingsManager.unitTestStore = store
    }

    override class func tearDown() {
        SettingsManager.unitTestStore = nil
    }

    override func setUpWithError() throws {
        let transportProvider = REST.AnyTransportProvider { nil }
        let addressCache = REST.AddressCache(canWriteToCache: false, fileCache: MemoryCache())
        let proxyConfiguration = REST.ProxyConfiguration(
            transportProvider: transportProvider,
            addressCacheStore: addressCache
        )
        let authProxy = REST.AuthProxyConfiguration(
            proxyConfiguration: proxyConfiguration,
            accessTokenManager: AccessTokenManagerStub()
        )

        proxyFactory = REST.ProxyFactory(configuration: authProxy)
        manager = MigrationManager()
    }

    func testNothingToMigrate() throws {
        let store = Self.store
        let settings = TunnelSettingsV3()
        try SettingsManager.writeSettings(settings)

        let nothingToMigrationExpectation = expectation(description: "No migration")
        manager.migrateSettings(store: store, proxyFactory: proxyFactory) { result in
            if case .nothing = result {
                nothingToMigrationExpectation.fulfill()
            }
        }
        wait(for: [nothingToMigrationExpectation], timeout: 1)
    }

    func testNothingToMigrateIfRecordedSettingsVersionHigherThanLatestSettings() throws {
        let store = Self.store
        let settings = FutureVersionSettings()
        try write(settings: settings, version: Int.max - 1, in: store)

        let nothingToMigrationExpectation = expectation(description: "No migration")
        manager.migrateSettings(store: store, proxyFactory: proxyFactory) { result in
            if case .nothing = result {
                nothingToMigrationExpectation.fulfill()
            }
        }
        wait(for: [nothingToMigrationExpectation], timeout: 1)
    }

    func testNothingToMigrateWhenSettingsAreNotFound() throws {
        let store = InMemorySettingsStore<KeychainError>()
        SettingsManager.unitTestStore = store

        let nothingToMigrationExpectation = expectation(description: "No migration")
        manager.migrateSettings(store: store, proxyFactory: proxyFactory) { result in
            if case .nothing = result {
                nothingToMigrationExpectation.fulfill()
            }
        }
        wait(for: [nothingToMigrationExpectation], timeout: 1)

        // Reset the `SettingsManager` unit test store to avoid affecting other tests
        // since it's a globally shared instance
        SettingsManager.unitTestStore = Self.store
    }

    func testFailedMigration() throws {
        let store = Self.store
        let failedMigrationExpectation = expectation(description: "Failed migration")
        manager.migrateSettings(store: store, proxyFactory: proxyFactory) { result in
            if case .failure = result {
                failedMigrationExpectation.fulfill()
            }
        }
        wait(for: [failedMigrationExpectation], timeout: 1)
    }

    func testFailedMigrationResetsSettings() throws {
        let store = Self.store
        let data = try XCTUnwrap("Migration test".data(using: .utf8))
        try store.write(data, for: .settings)
        try store.write(data, for: .deviceState)

        // Failed migration should reset settings and device state keys
        manager.migrateSettings(store: store, proxyFactory: proxyFactory) { _ in }

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
        manager.migrateSettings(store: store, proxyFactory: proxyFactory) { result in
            if case .failure = result {
                failedMigrationExpectation.fulfill()
            }
        }
        wait(for: [failedMigrationExpectation], timeout: 1)
    }

    func testSuccessfulMigrationFromV2ToLatest() throws {
        var settingsV2 = TunnelSettingsV2()
        let osakaRelayConstraints: RelayConstraints = .init(location: .only(.city("jp", "osa")))
        settingsV2.relayConstraints = osakaRelayConstraints

        try migrateToLatest(settingsV2, version: .v2)

        let latestSettings = try SettingsManager.readSettings()
        XCTAssertEqual(osakaRelayConstraints, latestSettings.relayConstraints)
    }

    func testSuccessfulMigrationFromV1ToLatest() throws {
        var settingsV1 = TunnelSettingsV1()
        let osakaRelayConstraints: RelayConstraints = .init(location: .only(.city("jp", "osa")))
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
        manager.migrateSettings(store: store, proxyFactory: proxyFactory) { result in
            if case .success = result {
                successfulMigrationExpectation.fulfill()
            }
        }
        wait(for: [successfulMigrationExpectation], timeout: 1)
    }

    private func write(settings: any TunnelSettings, version: Int, in store: SettingsStore) throws {
        let parser = SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
        let payload = try parser.producePayload(settings, version: version)
        try store.write(payload, for: .settings)
    }
}

private struct FutureVersionSettings: TunnelSettings {
    func upgradeToNextVersion(
        store: SettingsStore,
        proxyFactory: REST.ProxyFactory,
        parser: SettingsParser
    ) -> TunnelSettings { self }
}

struct SettingNotFound: Error, Instantiable {}

extension KeychainError: Instantiable {
    init() {
        self = KeychainError.itemNotFound
    }
}
