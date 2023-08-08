//
//  SettingsManager.swift
//  MullvadVPN
//
//  Created by pronebird on 29/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes

private let keychainServiceName = "Mullvad VPN"
private let accountTokenKey = "accountToken"
private let accountExpiryKey = "accountExpiry"

enum SettingsMigrationResult {
    /// Nothing to migrate.
    case nothing

    /// Successfully performed migration.
    case success

    /// Failure when migrating store.
    case failure(Error)
}

enum SettingsManager {
    private static let logger = Logger(label: "SettingsManager")

    private static let store: SettingsStore = KeychainSettingsStore(
        serviceName: keychainServiceName,
        accessGroup: ApplicationConfiguration.securityGroupIdentifier
    )

    private static func makeParser() -> SettingsParser {
        SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
    }

    // MARK: - Last used account

    static func getLastUsedAccount() throws -> String {
        let data = try store.read(key: .lastUsedAccount)

        if let string = String(data: data, encoding: .utf8) {
            return string
        } else {
            throw StringDecodingError(data: data)
        }
    }

    static func setLastUsedAccount(_ string: String?) throws {
        if let string {
            guard let data = string.data(using: .utf8) else {
                throw StringEncodingError(string: string)
            }

            try store.write(data, for: .lastUsedAccount)
        } else {
            do {
                try store.delete(key: .lastUsedAccount)
            } catch let error as KeychainError where error == .itemNotFound {
                return
            } catch {
                throw error
            }
        }
    }

    // MARK: - Should wipe settings

    static func getShouldWipeSettings() -> Bool {
        (try? store.read(key: .shouldWipeSettings)) != nil
    }

    static func setShouldWipeSettings() {
        do {
            try store.write(Data(), for: .shouldWipeSettings)
        } catch {
            logger.error(
                error: error,
                message: "Failed to set should wipe settings."
            )
        }
    }

    // MARK: - Settings

    static func readSettings() throws -> LatestTunnelSettings {
        let storedVersion: Int
        let data: Data
        let parser = makeParser()

        do {
            data = try store.read(key: .settings)
            storedVersion = try parser.parseVersion(data: data)
        } catch {
            throw ReadSettingsVersionError(underlyingError: error)
        }

        let currentVersion = SchemaVersion.current

        if storedVersion == currentVersion.rawValue {
            return try parser.parsePayload(as: LatestTunnelSettings.self, from: data)
        } else {
            throw UnsupportedSettingsVersionError(
                storedVersion: storedVersion,
                currentVersion: currentVersion
            )
        }
    }

    static func writeSettings(_ settings: LatestTunnelSettings) throws {
        let parser = makeParser()
        let data = try parser.producePayload(settings, version: SchemaVersion.current.rawValue)

        try store.write(data, for: .settings)
    }

    // MARK: - Device state

    static func readDeviceState() throws -> DeviceState {
        let data = try store.read(key: .deviceState)
        let parser = makeParser()

        return try parser.parseUnversionedPayload(as: DeviceState.self, from: data)
    }

    static func writeDeviceState(_ deviceState: DeviceState) throws {
        let parser = makeParser()
        let data = try parser.produceUnversionedPayload(deviceState)

        try store.write(data, for: .deviceState)
    }

    // MARK: - Migration

    /// Migrate settings store if needed.
    ///
    /// The following types of error are expected to be returned by this method:
    /// `SettingsMigrationError`, `UnsupportedSettingsVersionError`, `ReadSettingsVersionError`.
    static func migrateStore(
        with restFactory: REST.ProxyFactory,
        completion: @escaping (SettingsMigrationResult) -> Void
    ) {
        let handleCompletion = { (result: SettingsMigrationResult) in
            // Reset store upon failure to migrate settings.
            if case .failure = result {
                resetStore()
            }
            completion(result)
        }

        do {
            try checkLatestSettingsVersion()
            handleCompletion(.nothing)
        } catch {
            handleCompletion(.failure(error))
        }
    }

    /// Removes all legacy settings, device state and tunnel settings but keeps the last used
    /// account number stored.
    static func resetStore(completely: Bool = false) {
        logger.debug("Reset store.")

        do {
            try store.delete(key: .deviceState)
        } catch {
            if (error as? KeychainError) != .itemNotFound {
                logger.error(error: error, message: "Failed to delete device state.")
            }
        }

        do {
            try store.delete(key: .settings)
        } catch {
            if (error as? KeychainError) != .itemNotFound {
                logger.error(error: error, message: "Failed to delete settings.")
            }
        }

        if completely {
            do {
                try store.delete(key: .lastUsedAccount)
            } catch {
                if (error as? KeychainError) != .itemNotFound {
                    logger.error(error: error, message: "Failed to delete last used account.")
                }
            }

            do {
                try store.delete(key: .shouldWipeSettings)
            } catch {
                if (error as? KeychainError) != .itemNotFound {
                    logger.error(error: error, message: "Failed to delete should wipe settings.")
                }
            }
        }
    }

    // MARK: - Private

    private static func checkLatestSettingsVersion() throws {
        let settingsVersion: Int
        do {
            let parser = makeParser()
            let settingsData = try store.read(key: .settings)
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

enum SettingsKey: String, CaseIterable {
    case settings = "Settings"
    case deviceState = "DeviceState"
    case lastUsedAccount = "LastUsedAccount"
    case shouldWipeSettings = "ShouldWipeSettings"
}

/// An error type describing a failure to read or parse settings version.
struct ReadSettingsVersionError: LocalizedError, WrappingError {
    private let inner: Error

    var underlyingError: Error? {
        inner
    }

    var errorDescription: String? {
        "Failed to read settings version."
    }

    init(underlyingError: Error) {
        inner = underlyingError
    }
}

/// An error returned when stored settings version is unknown to the currently running app.
struct UnsupportedSettingsVersionError: LocalizedError {
    let storedVersion: Int
    let currentVersion: SchemaVersion

    var errorDescription: String? {
        """
        Stored settings version was not the same as current version, \
        stored version: \(storedVersion), current version: \(currentVersion)
        """
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

struct StringDecodingError: LocalizedError {
    let data: Data

    var errorDescription: String? {
        "Failed to decode string from data."
    }
}

struct StringEncodingError: LocalizedError {
    let string: String

    var errorDescription: String? {
        "Failed to encode string into data."
    }
}
