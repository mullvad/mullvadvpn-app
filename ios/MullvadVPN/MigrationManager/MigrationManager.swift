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

enum SettingsMigrationResult {
    /// Nothing to migrate.
    case nothing

    /// Successfully performed migration.
    case success

    /// Failure when migrating store.
    case failure(Error)
}

struct MigrationManager {
    private let logger = Logger(label: "MigrationManager")

    /// Migrate settings store if needed.
    ///
    /// The following types of error are expected to be returned by this method:
    /// `SettingsMigrationError`, `UnsupportedSettingsVersionError`, `ReadSettingsVersionError`.
    func migrateSettings(
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
        } catch {
            handleCompletion(.failure(error))
        }
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
struct SettingsMigrationError: LocalizedError, WrappingError {
    private let inner: Error
    let sourceVersion, targetVersion: SchemaVersion

    var underlyingError: Error? {
        inner
    }

    var errorDescription: String? {
        "Failed to migrate settings from \(sourceVersion) to \(targetVersion)."
    }

    init(sourceVersion: SchemaVersion, targetVersion: SchemaVersion, underlyingError: Error) {
        self.sourceVersion = sourceVersion
        self.targetVersion = targetVersion
        inner = underlyingError
    }
}
