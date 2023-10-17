//
//  MigrationManager.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-08-08.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes

public enum SettingsMigrationResult {
    /// Nothing to migrate.
    case nothing

    /// Successfully performed migration.
    case success

    /// Failure when migrating store.
    case failure(Error)
}

public struct MigrationManager {
    private let logger = Logger(label: "MigrationManager")

    public init() {}

    /// Migrate settings store if needed.
    ///
    /// Reads the current settings, upgrades them to the latest version if needed
    /// and writes back to `store` when settings are updated.
    /// - Parameters:
    ///   - store: The store to from which settings are read and written to.
    ///   - proxyFactory: Factory used for migrations that involve API calls.
    ///   - migrationCompleted: Completion handler called with a migration result.
    public func migrateSettings(
        store: SettingsStore,
        proxyFactory: REST.ProxyFactory,
        migrationCompleted: @escaping (SettingsMigrationResult) -> Void
    ) {
        let resetStoreHandler = { (result: SettingsMigrationResult) in
            // Reset store upon failure to migrate settings.
            if case .failure = result {
                SettingsManager.resetStore()
            }
            migrationCompleted(result)
        }

        do {
            try upgradeSettingsToLatestVersion(
                store: store,
                proxyFactory: proxyFactory,
                migrationCompleted: migrationCompleted
            )
        } catch .itemNotFound as KeychainError {
            migrationCompleted(.nothing)
        } catch {
            resetStoreHandler(.failure(error))
        }
    }

    private func upgradeSettingsToLatestVersion(
        store: SettingsStore,
        proxyFactory: REST.ProxyFactory,
        migrationCompleted: @escaping (SettingsMigrationResult) -> Void
    ) throws {
        let parser = SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
        let settingsData = try store.read(key: SettingsKey.settings)
        let settingsVersion = try parser.parseVersion(data: settingsData)

        guard settingsVersion != SchemaVersion.current.rawValue else {
            migrationCompleted(.nothing)
            return
        }

        // Corrupted settings version (i.e. negative values, or downgrade from a future version) should fail
        guard var savedSchema = SchemaVersion(rawValue: settingsVersion) else {
            migrationCompleted(.failure(UnsupportedSettingsVersionError(
                storedVersion: settingsVersion,
                currentVersion: SchemaVersion.current
            )))
            return
        }

        var savedSettings = try parser.parsePayload(as: savedSchema.settingsType, from: settingsData)

        repeat {
            let upgradedVersion = savedSettings.upgradeToNextVersion()
            savedSchema = savedSchema.nextVersion
            savedSettings = upgradedVersion
        } while savedSchema.rawValue < SchemaVersion.current.rawValue

        // Write the latest settings back to the store
        let latestVersionPayload = try parser.producePayload(savedSettings, version: SchemaVersion.current.rawValue)
        try store.write(latestVersionPayload, for: .settings)
        migrationCompleted(.success)
    }
}
