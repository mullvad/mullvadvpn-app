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

enum SettingsManager {}

struct LegacyTunnelSettings {
    let accountNumber: String
    let tunnelSettings: TunnelSettingsV1
}

private let keychainServiceName = "Mullvad VPN"
private let accountTokenKey = "accountToken"
private let accountExpiryKey = "accountExpiry"

enum SettingsStorableItem: String, CaseIterable {
    case settings = "Settings"
    case deviceState = "DeviceState"
    case lastUsedAccount = "LastUsedAccount"
}

protocol SettingsStore: AnyObject {
    func read(for itemKey: SettingsStorableItem) throws -> Data
    func write(_ data: Data, for itemKey: SettingsStorableItem) throws
    func delete(itemKey: SettingsStorableItem) throws
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

extension SettingsManager {
    private static func makeDecoder() -> JSONDecoder {
        JSONDecoder()
    }

    private static func makeEncoder() -> JSONEncoder {
        JSONEncoder()
    }

    // MARK: - Lsat used account

    static func getLastUsedAccount() throws -> String {
        let data = try store.read(for: .lastUsedAccount)

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
                try store.delete(itemKey: .lastUsedAccount)
            } catch let error as KeychainError where error == .itemNotFound {
                return
            } catch {
                throw error
            }
        }
    }

    private static let store: SettingsStore = KeychainSettingsFacade()

    // MARK: - Settings

    static func readSettings() throws -> TunnelSettingsV2 {
        let readerWriter = makeStorageMiddlewareFactory()

        return try readerWriter.readSettings()
    }

    static func writeSettings(_ settings: TunnelSettingsV2) throws {
        let readerWriter = makeStorageMiddlewareFactory()

        return try readerWriter.saveSettings(settings)
    }

    static func deleteSettings() throws {
        try store.delete(itemKey: .settings)
    }

    // MARK: - Device state

    static func readDeviceState() throws -> DeviceState {
        let readerWriter = makeStorageMiddlewareFactory()

        return try readerWriter.readDeviceState()
    }

    static func writeDeviceState(_ deviceState: DeviceState) throws {
        let readerWriter = makeStorageMiddlewareFactory()

        return try readerWriter.saveDeviceState(deviceState)
    }

    static func deleteDeviceState() throws {
        try store.delete(itemKey: .deviceState)
    }

    // MARK: - Migration

    static func makeStorageMiddlewareFactory(
        store: SettingsStore = Self.store,
        decoder: JSONDecoder = Self.makeDecoder(),
        encoder: JSONEncoder = Self.makeEncoder()
    ) -> SettingsStorageMiddleware {
        SettingsStorageMiddleware(
            store: store,
            decoder: decoder,
            encoder: encoder
        )
    }

    static func migrateStore(
        with restFactory: REST.ProxyFactory,
        completion: @escaping (Error?) -> Void
    ) {
        guard let settingsData = try? store.read(for: .settings),
              let deviceStateData = try? store.read(for: .deviceState)
        else {
            // Return new/not logged in user immediately.
            completion(nil)
            return
        }

        // Check versions.
        let decoder = Self.makeDecoder()

        if let settingsHeader = try? decoder.decode(VersionHeader.self, from: settingsData),
           let deviceStateHeader = try? decoder.decode(VersionHeader.self, from: deviceStateData)
        {
            if settingsHeader.version != SchemaVersion.current.rawValue {
                completion(VersioningError.unsupportedSettings(
                    storedVersion: settingsHeader.version,
                    currentVersion: SchemaVersion.current
                        .rawValue
                ))

                return
            }

            if deviceStateHeader.version != SchemaVersion.current.rawValue {
                completion(VersioningError.unsupportedDeviceState(
                    storedVersion: deviceStateHeader.version,
                    currentVersion: SchemaVersion.current
                        .rawValue
                ))

                return
            }

            completion(nil)
            return
        }

        let storageMiddleware = Self.makeStorageMiddlewareFactory()

        // Check for legacy settings.
        if let legacySettings = readLegacySettings() {
            let migration = MigrationFromV1ToV2(
                restFactory: restFactory,
                legacySettings: legacySettings,
                logger: logger
            )

            migration.migrate(with: storageMiddleware) { error in
                if let error = error {
                    completion(error)
                } else {
                    // migration was successful, deleting legacy settings.
                    let userDefaults = UserDefaults.standard

                    logger.debug("Remove legacy settings from keychain.")
                    Self.deleteLegacySettings()

                    logger.debug("Remove legacy settings from user defaults.")

                    userDefaults.removeObject(forKey: accountTokenKey)
                    userDefaults.removeObject(forKey: accountExpiryKey)

                    completion(nil)
                }
            }

            return
        }

        // Check for unversion settings.
        let migrator = MigrationFromUnversionedToV2(
            settingsData: settingsData,
            deviceStateData: deviceStateData,
            logger: logger
        )

        migrator.migrate(with: storageMiddleware) { error in
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

        return SettingsStorableItem(rawValue: accountNumber) == nil
    }
}

// MARK: - Keychain Facade

class KeychainSettingsFacade: SettingsStore {
    func read(for itemKey: SettingsStorableItem) throws -> Data {
        try readItemData(itemKey)
    }

    func write(_ data: Data, for itemKey: SettingsStorableItem) throws {
        try addOrUpdateItem(itemKey, data: data)
    }

    func delete(itemKey: SettingsStorableItem) throws {
        try deleteItem(itemKey)
    }

    private func addItem(_ item: SettingsStorableItem, data: Data) throws {
        var query = createDefaultAttributes(item: item)
        query.merge(createAccessAttributes()) { current, _ in
            return current
        }
        query[kSecValueData] = data

        let status = SecItemAdd(query as CFDictionary, nil)
        if status != errSecSuccess {
            throw KeychainError(code: status)
        }
    }

    private func updateItem(_ item: SettingsStorableItem, data: Data) throws {
        let query = createDefaultAttributes(item: item)
        let status = SecItemUpdate(
            query as CFDictionary,
            [kSecValueData: data] as CFDictionary
        )

        if status != errSecSuccess {
            throw KeychainError(code: status)
        }
    }

    private func addOrUpdateItem(_ item: SettingsStorableItem, data: Data) throws {
        do {
            try updateItem(item, data: data)
        } catch let error as KeychainError where error == .itemNotFound {
            try addItem(item, data: data)
        } catch {
            throw error
        }
    }

    private func readItemData(_ item: SettingsStorableItem) throws -> Data {
        var query = createDefaultAttributes(item: item)
        query[kSecReturnData] = true

        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        if status == errSecSuccess {
            return result as? Data ?? Data()
        } else {
            throw KeychainError(code: status)
        }
    }

    private func deleteItem(_ item: SettingsStorableItem) throws {
        let query = createDefaultAttributes(item: item)
        let status = SecItemDelete(query as CFDictionary)
        if status != errSecSuccess {
            throw KeychainError(code: status)
        }
    }

    private func createDefaultAttributes(item: SettingsStorableItem) -> [CFString: Any] {
        return [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: keychainServiceName,
            kSecAttrAccount: item.rawValue,
        ]
    }

    private func createAccessAttributes() -> [CFString: Any] {
        return [
            kSecAttrAccessGroup: ApplicationConfiguration.securityGroupIdentifier,
            kSecAttrAccessible: kSecAttrAccessibleAfterFirstUnlock,
        ]
    }
}

// MARK: - Settings Storage Middleware

private struct VersionHeader: Codable {
    var version: Int
}

struct SettingsStorageMiddleware {
    private struct Payload<T: Codable>: Codable {
        var data: T
    }

    private struct VersionedPayload<T: Codable>: Codable {
        var version: Int
        var data: T
    }

    /// Top level data storage.
    private let store: SettingsStore

    /// The decoder used to decode values.
    private let decoder: JSONDecoder

    /// The encoder used to encode values.
    private let encoder: JSONEncoder

    fileprivate init(store: SettingsStore, decoder: JSONDecoder, encoder: JSONEncoder) {
        self.store = store
        self.decoder = decoder
        self.encoder = encoder
    }

    func readSettings() throws -> TunnelSettingsV2 {
        let data = try store.read(for: .settings)

        return try read(data: data)
    }

    func readDeviceState() throws -> DeviceState {
        let data = try store.read(for: .deviceState)

        return try read(data: data)
    }

    /// Returns unversioned payload parsed as the given type.
    func parseUnversionedPayload<T: Codable>(
        as type: T.Type,
        from data: Data
    ) throws -> T {
        return try decoder.decode(T.self, from: data)
    }

    /// Persist versioned `TunnelSettings` payload to the keychain store.
    func saveSettings(_ payload: TunnelSettingsV2) throws {
        let versionedPayload = VersionedPayload(
            version: SchemaVersion.current.rawValue,
            data: payload
        )

        let data = try encoder.encode(versionedPayload)

        try store.write(data, for: .settings)
    }

    /// Persist versioned `DeviceState` payload to the keychain store.
    func saveDeviceState(_ payload: DeviceState) throws {
        let versionedPayload = VersionedPayload(
            version: SchemaVersion.current.rawValue,
            data: payload
        )

        let data = try encoder.encode(versionedPayload)

        try store.write(data, for: .deviceState)
    }

    // Private

    /// Returns payload data for the given type.
    private func read<T: Codable>(data: Data) throws -> T {
        let storedVersion = try parseVersion(data: data)
        let currentVersionNumber = SchemaVersion.current.rawValue

        if storedVersion != currentVersionNumber {
            throw VersioningError.unsupportedSettings(
                storedVersion: storedVersion,
                currentVersion: currentVersionNumber
            )
        }

        return try parsePayload(as: T.self, from: data).data
    }

    /// Returns settings version if found inside the stored data.
    private func parseVersion(data: Data) throws -> Int {
        let header = try decoder.decode(VersionHeader.self, from: data)

        return header.version
    }

    /// Returns payload type holding the given type.
    private func parsePayload<T: Codable>(
        as type: T.Type,
        from data: Data
    ) throws -> Payload<T> {
        return try decoder.decode(Payload<T>.self, from: data)
    }
}

// MARK: - Versioning Error

/// An error type that contains description about version handling.
enum VersioningError: LocalizedError {
    /// Difference between stored `TunnelSettings` version and current version
    case unsupportedSettings(storedVersion: Int, currentVersion: Int)

    /// Difference between stored `DeviceState` version and current version
    case unsupportedDeviceState(storedVersion: Int, currentVersion: Int)

    var errorDescription: String? {
        switch self {
        case let .unsupportedSettings(storedVersion, currentVersion):
            return "Stored settings version was not the same as current version, stored version: \(storedVersion), current version: \(currentVersion)"
        case let .unsupportedDeviceState(storedVersion, currentVersion):
            return "Stored device state version was not the same as current version, stored version: \(storedVersion), current version: \(currentVersion)"
        }
    }
}
