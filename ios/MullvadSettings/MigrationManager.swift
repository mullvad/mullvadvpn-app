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
    /// The following types of error are expected to be returned by this method:
    /// `SettingsMigrationError`, `UnsupportedSettingsVersionError`, `ReadSettingsVersionError`.
    public func migrateSettings(
        store: SettingsStore,
        proxyFactory: REST.ProxyFactory,
        migrationCompleted: @escaping (SettingsMigrationResult) -> Void
    ) {
        let handleCompletion = { (result: SettingsMigrationResult) in
            // Reset store upon failure to migrate settings.
            if case .failure = result {
                SettingsManager.resetStore()
            }
            migrationCompleted(result)
        }

        do {
            try checkLatestSettingsVersion(in: store)
            handleCompletion(.nothing)
        } catch is UnsupportedSettingsVersionError {
            do {
                try upgradeSettingsToLatestVersion(
                    store: store,
                    proxyFactory: proxyFactory,
                    migrationCompleted: migrationCompleted
                )
            } catch {
                handleCompletion(.failure(error))
            }
        } catch {
            handleCompletion(.failure(error))
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

        // Special case downgrade attempts as nothing to do
        guard settingsVersion <= SchemaVersion.current.rawValue else {
            migrationCompleted(.nothing)
            return
        }

        // Handle cases where the saved version is strictly inferior to the current version
        guard let savedSchema = SchemaVersion(rawValue: settingsVersion) else {
            migrationCompleted(.failure(UnsupportedSettingsVersionError(
                storedVersion: settingsVersion,
                currentVersion: SchemaVersion.current
            )))
            return
        }

        var versionTypeCopy = savedSchema
        let savedSettings = try parser.parsePayload(as: versionTypeCopy.settingsType, from: settingsData)
        var latestSettings = savedSettings

        repeat {
            let upgradedVersion = latestSettings.upgradeToNextVersion(
                store: store,
                proxyFactory: proxyFactory,
                parser: parser
            )
            versionTypeCopy = versionTypeCopy.nextVersion
            latestSettings = upgradedVersion
        } while versionTypeCopy.rawValue < SchemaVersion.current.rawValue

        // Write the latest settings back to the store
        let latestVersionPayload = try parser.producePayload(latestSettings, version: SchemaVersion.current.rawValue)
        try store.write(latestVersionPayload, for: .settings)
        migrationCompleted(.success)
    }

    private func checkLatestSettingsVersion(in store: SettingsStore) throws {
        let settingsVersion: Int
        do {
            let parser = SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
            let settingsData = try store.read(key: SettingsKey.settings)
            settingsVersion = try parser.parseVersion(data: settingsData)
        } catch .itemNotFound as KeychainError {
            return
        } catch {
            throw ReadSettingsVersionError(underlyingError: error)
        }

        guard settingsVersion != SchemaVersion.current.rawValue else {
            return
        }

        let error = UnsupportedSettingsVersionError(
            storedVersion: settingsVersion,
            currentVersion: SchemaVersion.current
        )

        logger.error(error: error, message: "Encountered an unknown version.")

        throw error
    }
}

/// A wrapper type for errors returned by concrete migrations.
public struct SettingsMigrationError: LocalizedError, WrappingError {
    private let inner: Error
    public let sourceVersion, targetVersion: SchemaVersion

    public var underlyingError: Error? {
        inner
    }

    public var errorDescription: String? {
        "Failed to migrate settings from \(sourceVersion) to \(targetVersion)."
    }

    public init(sourceVersion: SchemaVersion, targetVersion: SchemaVersion, underlyingError: Error) {
        self.sourceVersion = sourceVersion
        self.targetVersion = targetVersion
        inner = underlyingError
    }
}
