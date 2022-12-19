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
        return SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
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
        if let string = string {
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

    // MARK: - Settings

    static func readSettings() throws -> TunnelSettingsV2 {
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
            return try parser.parsePayload(as: TunnelSettingsV2.self, from: data)
        } else {
            throw UnsupportedSettingsVersionError(
                storedVersion: storedVersion,
                currentVersion: currentVersion
            )
        }
    }

    static func writeSettings(_ settings: TunnelSettingsV2) throws {
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
                self.resetStore()
            }
            completion(result)
        }

        if let legacySettings = readLegacySettings() {
            migrateLegacySettings(
                restFactory: restFactory,
                legacySettings: legacySettings
            ) { error in
                handleCompletion(error.map { .failure($0) } ?? .success)
            }
        } else {
            do {
                try checkLatestSettingsVersion()

                handleCompletion(.nothing)
            } catch {
                handleCompletion(.failure(error))
            }
        }
    }

    // MARK: - Private

    private static func migrateLegacySettings(
        restFactory: REST.ProxyFactory,
        legacySettings: LegacyTunnelSettings,
        completion: @escaping (Error?) -> Void
    ) {
        let parser = makeParser()

        let migration = MigrationFromV1ToV2(
            restFactory: restFactory,
            legacySettings: legacySettings
        )

        migration.migrate(with: store, parser: parser) { error in
            if let error = error {
                let migrationError = SettingsMigrationError(
                    sourceVersion: .v1,
                    targetVersion: .v2,
                    underlyingError: error
                )

                logger.error(error: migrationError)

                completion(migrationError)
            } else {
                Self.deleteAllLegacySettings()

                completion(nil)
            }
        }
    }

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

    /// Removes all legacy settings, device state and tunnel settings but keeps the last used
    /// account number stored.
    private static func resetStore() {
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

        Self.deleteAllLegacySettings()
    }

    // MARK: - Legacy settings

    private static func readLegacySettings() -> LegacyTunnelSettings? {
        guard let storedAccountNumber = UserDefaults.standard.string(forKey: accountTokenKey) else {
            logger.debug("Legacy account number is not found in user defaults. Nothing to migrate.")
            return nil
        }

        // List legacy settings stored in keychain.
        logger.debug("List legacy settings in keychain...")

        var storedSettings: [LegacyTunnelSettings] = []
        do {
            storedSettings = try findAllLegacySettingsInKeychain()
        } catch .itemNotFound as KeychainError {
            logger.debug("Legacy settings are not found in keychain.")

            return nil
        } catch {
            logger.error(
                error: error,
                message: "Failed to read legacy settings from keychain."
            )

            return nil
        }

        // Find settings matching the account number stored in user defaults.
        let matchingSettings = storedSettings.first { settings in
            return settings.accountNumber == storedAccountNumber
        }

        guard let matchingSettings = matchingSettings else {
            logger.debug(
                "Could not find legacy settings matching the legacy account number."
            )

            return nil
        }

        return matchingSettings
    }

    private static func findAllLegacySettingsInKeychain() throws -> [LegacyTunnelSettings] {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: keychainServiceName,
            kSecReturnAttributes: true,
            kSecReturnData: true,
            kSecMatchLimit: kSecMatchLimitAll,
        ]

        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        guard status == errSecSuccess else {
            throw KeychainError(code: status)
        }

        guard let items = result as? [[CFString: Any]] else {
            return []
        }

        return items.filter(Self.filterLegacySettings)
            .compactMap { item -> LegacyTunnelSettings? in
                guard let accountNumber = item[kSecAttrAccount] as? String,
                      let data = item[kSecValueData] as? Data
                else {
                    return nil
                }
                do {
                    let tunnelSettings = try JSONDecoder().decode(
                        TunnelSettingsV1.self,
                        from: data
                    )

                    return LegacyTunnelSettings(
                        accountNumber: accountNumber,
                        tunnelSettings: tunnelSettings
                    )
                } catch {
                    logger.error(
                        error: error,
                        message: "Failed to decode legacy settings."
                    )
                    return nil
                }
            }
    }

    private static func deleteAllLegacySettings() {
        logger.debug("Remove legacy settings from keychain.")
        deleteLegacySettingsFromKeychain()

        logger.debug("Remove legacy settings from user defaults.")
        let userDefaults = UserDefaults.standard
        userDefaults.removeObject(forKey: accountTokenKey)
        userDefaults.removeObject(forKey: accountExpiryKey)
    }

    private static func deleteLegacySettingsFromKeychain() {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: keychainServiceName,
            kSecReturnAttributes: true,
            kSecMatchLimit: kSecMatchLimitAll,
        ]

        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        guard status == errSecSuccess else {
            let error = KeychainError(code: status)

            if error != .itemNotFound {
                logger.error(
                    error: error,
                    message: "Failed to list legacy settings."
                )
            }

            return
        }

        guard let items = result as? [[CFString: Any]] else {
            return
        }

        items.filter(Self.filterLegacySettings)
            .enumerated()
            .forEach { index, item in
                guard let account = item[kSecAttrAccount] else {
                    return
                }

                let deleteQuery: [CFString: Any] = [
                    kSecClass: kSecClassGenericPassword,
                    kSecAttrService: keychainServiceName,
                    kSecAttrAccount: account,
                ]

                let status = SecItemDelete(deleteQuery as CFDictionary)
                if status == errSecSuccess {
                    logger.debug("Removed legacy settings entry \(index).")
                } else {
                    let error = KeychainError(code: status)

                    logger.error(
                        error: error,
                        message: "Failed to remove legacy settings entry \(index)."
                    )
                }
            }
    }

    private static func filterLegacySettings(_ item: [CFString: Any]) -> Bool {
        guard let accountNumber = item[kSecAttrAccount] as? String else {
            return false
        }

        return SettingsKey(rawValue: accountNumber) == nil
    }
}

struct LegacyTunnelSettings {
    let accountNumber: String
    let tunnelSettings: TunnelSettingsV1
}

enum SettingsKey: String, CaseIterable {
    case settings = "Settings"
    case deviceState = "DeviceState"
    case lastUsedAccount = "LastUsedAccount"
}

/// An error type describing a failure to read or parse settings version.
struct ReadSettingsVersionError: LocalizedError, WrappingError {
    private let inner: Error

    var underlyingError: Error? {
        return inner
    }

    var errorDescription: String? {
        return "Failed to read settings version."
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
        return """
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
        return inner
    }

    var errorDescription: String? {
        return "Failed to migrate settings from \(sourceVersion) to \(targetVersion)."
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
        return "Failed to decode string from data."
    }
}

struct StringEncodingError: LocalizedError {
    let string: String

    var errorDescription: String? {
        return "Failed to encode string into data."
    }
}
