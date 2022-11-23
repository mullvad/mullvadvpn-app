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

enum SettingsManager {
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

    private static let store: SettingsStore = KeychainSettingsStore(
        keychainServiceName: keychainServiceName
    )

    // MARK: - Settings

    static func readSettings() throws -> TunnelSettingsV2 {
        let data = try store.read(key: .settings)
        let parser = makeParser()

        let version = try parser.parseVersion(data: data)
        let currentVersion = SchemaVersion.current.rawValue

        if version == currentVersion {
            return try parser.parsePayload(as: TunnelSettingsV2.self, from: data)
        } else {
            throw UnsupportedVersionSettings(storedVersion: version, currentVersion: currentVersion)
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

    static func migrateStore(
        with restFactory: REST.ProxyFactory,
        completion: @escaping (Error?) -> Void
    ) {
        if let legacySettings = readLegacySettings() {
            migrateLegacySettings(
                restFactory: restFactory,
                legacySettings: legacySettings,
                completion: completion
            )
        } else {
            migrateModernSettings(completion: completion)
        }
    }

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
                logger.error(
                    error: error,
                    message: "Failed to migrate from legacy settings to v2."
                )

                completion(error)
            } else {
                let userDefaults = UserDefaults.standard

                logger.debug("Remove legacy settings from keychain.")
                Self.deleteLegacySettings()

                logger.debug("Remove legacy settings from user defaults.")

                userDefaults.removeObject(forKey: accountTokenKey)
                userDefaults.removeObject(forKey: accountExpiryKey)

                completion(nil)
            }
        }
    }

    private static func migrateModernSettings(completion: @escaping (Error?) -> Void) {
        let parser = makeParser()

        do {
            let settingsData = try store.read(key: .settings)
            let settingsVersion = try parser.parseVersion(data: settingsData)

            if settingsVersion != SchemaVersion.current.rawValue {
                let error = UnsupportedVersionSettings(
                    storedVersion: settingsVersion,
                    currentVersion: SchemaVersion.current.rawValue
                )

                logger.error(
                    error: error,
                    message: "Encountered an unknown version."
                )

                completion(error)

            } else {
                completion(nil)
            }

        } catch .itemNotFound as KeychainError {
            completion(nil)
        } catch {
            completion(error)
        }
    }

    // MARK: - Legacy settings

    private static func readLegacySettings() -> LegacyTunnelSettings? {
        let storedAccountNumber = UserDefaults.standard.string(forKey: accountTokenKey)

        guard let storedAccountNumber = storedAccountNumber else {
            logger.debug("Account number is not found in user defaults. Nothing to migrate.")

            return nil
        }

        // Set legacy account number as last used.
        logger.debug("Found legacy account number.")
        logger.debug("Store last used account.")

        do {
            try Self.setLastUsedAccount(storedAccountNumber)
        } catch {
            logger.error(
                error: error,
                message: "Failed to store last used account."
            )
        }

        // List legacy settings stored in keychain.
        logger.debug("Read legacy settings...")

        var storedSettings: [LegacyTunnelSettings] = []
        do {
            storedSettings = try Self.readLegacySettings()
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

    // MARK: - Legacy settings support

    private static let logger = Logger(label: "SettingsManager")

    static func readLegacySettings() throws -> [LegacyTunnelSettings] {
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

    static func deleteLegacySettings() {
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

struct UnsupportedVersionSettings: LocalizedError {
    let storedVersion, currentVersion: Int

    var errorDescription: String? {
        return "Stored settings version was not the same as current version, stored version: \(storedVersion), current version: \(currentVersion)"
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
