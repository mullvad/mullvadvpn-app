//
//  SettingsManager.swift
//  MullvadVPN
//
//  Created by pronebird on 29/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes

enum SettingsManager {}

struct LegacyTunnelSettings {
    let accountNumber: String
    let tunnelSettings: TunnelSettingsV1
}

private let keychainServiceName = "Mullvad VPN"

private enum Item: String, CaseIterable {
    case settings = "Settings"
    case deviceState = "DeviceState"
    case lastUsedAccount = "LastUsedAccount"
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
    // MARK: - Lsat used account

    static func getLastUsedAccount() throws -> String {
        let data = try readItemData(.lastUsedAccount)

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

            try addOrUpdateItem(.lastUsedAccount, data: data)
        } else {
            do {
                try deleteItem(.lastUsedAccount)
            } catch let error as KeychainError where error == .itemNotFound {
                return
            } catch {
                throw error
            }
        }
    }

    // MARK: - Settings

    static func readSettings() throws -> TunnelSettingsV2 {
        let data = try readItemData(.settings)
        let version = try JSONDecoder().decode(VersionHeader.self, from: data).version

        if version != SchemaVersion.current.rawValue {
            throw VersioningError.unsupportedSettings(
                storedVersion: version,
                currentVersion: SchemaVersion.current.rawValue
            )
        }

        return try JSONDecoder().decode(Payload<TunnelSettingsV2>.self, from: data).data
    }

    static func writeSettings(_ settings: TunnelSettingsV2) throws {
        let versionedSettings = VersionedPayload(
            version: SchemaVersion.current.rawValue,
            data: settings
        )
        let data = try JSONEncoder().encode(versionedSettings)

        try addOrUpdateItem(.settings, data: data)
    }

    static func deleteSettings() throws {
        try deleteItem(.settings)
    }

    // MARK: - Device state

    static func readDeviceState() throws -> DeviceState {
        let data = try readItemData(.deviceState)
        let version = try JSONDecoder().decode(VersionHeader.self, from: data).version

        if version != SchemaVersion.current.rawValue {
            throw VersioningError.unsupportedDeviceState(
                storedVersion: version,
                currentVersion: SchemaVersion.current.rawValue
            )
        }

        return try JSONDecoder().decode(Payload<DeviceState>.self, from: data).data
    }

    static func writeDeviceState(_ deviceState: DeviceState) throws {
        let versionedDeviceData = VersionedPayload(
            version: SchemaVersion.current.rawValue,
            data: deviceState
        )
        let data = try JSONEncoder().encode(versionedDeviceData)

        try addOrUpdateItem(.deviceState, data: data)
    }

    static func deleteDeviceState() throws {
        try deleteItem(.deviceState)
    }

    // MARK: - Versioning

    private struct VersionHeader: Codable {
        var version: Int
    }

    struct Payload<T: Codable>: Codable {
        var data: T
    }

    struct VersionedPayload<T: Codable>: Codable {
        var version: Int
        var data: T
    }

    // MARK: - Migration manager

    static func tryMigrateSettings() throws -> MigrationManager? {
        return try tryMigrate(item: .settings)
    }

    static func tryMigrateDeviceState() throws -> MigrationManager? {
        return try tryMigrate(item: .deviceState)
    }

    private static func tryMigrate(item: Item) throws -> MigrationManager? {
        do {
            let data = try readItemData(item)
            let header = try? JSONDecoder().decode(VersionHeader.self, from: data)

            if header?.version == SchemaVersion.current.rawValue {
                return nil
            } else {
                return MigrationManager(item: item, data: data)
            }
        } catch .itemNotFound as KeychainError {
            return nil
        } catch {
            throw error
        }
    }

    struct MigrationManager {
        private let item: Item
        private var data: Data

        fileprivate init(item: Item, data: Data) {
            self.item = item
            self.data = data
        }

        /// Returns settings version if found inside the stored data.
        func parseVersion() -> Int? {
            let header = try? JSONDecoder().decode(VersionHeader.self, from: data)

            return header?.version
        }

        /// Returns payload type holding the given type.
        func parsePayload<T: Codable>(as type: T.Type) throws -> Payload<T> {
            return try JSONDecoder().decode(Payload<T>.self, from: data)
        }

        /// Returns unversioned payload parsed as the given type.
        func parseUnversionedPayload<T: Codable>(as type: T.Type) throws -> T {
            return try JSONDecoder().decode(T.self, from: data)
        }

        /// Persist versioned payload to the keychain store.
        mutating func store<T: Codable>(versionedPayload: VersionedPayload<T>) throws {
            let data = try JSONEncoder().encode(versionedPayload)

            try SettingsManager.addOrUpdateItem(item, data: data)

            self.data = data
        }
    }

    // MARK: - Keychain helpers

    private static func addItem(_ item: Item, data: Data) throws {
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

    private static func updateItem(_ item: Item, data: Data) throws {
        let query = createDefaultAttributes(item: item)
        let status = SecItemUpdate(
            query as CFDictionary,
            [kSecValueData: data] as CFDictionary
        )

        if status != errSecSuccess {
            throw KeychainError(code: status)
        }
    }

    private static func addOrUpdateItem(_ item: Item, data: Data) throws {
        do {
            try updateItem(item, data: data)
        } catch let error as KeychainError where error == .itemNotFound {
            try addItem(item, data: data)
        } catch {
            throw error
        }
    }

    private static func readItemData(_ item: Item) throws -> Data {
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

    private static func deleteItem(_ item: Item) throws {
        let query = createDefaultAttributes(item: item)
        let status = SecItemDelete(query as CFDictionary)
        if status != errSecSuccess {
            throw KeychainError(code: status)
        }
    }

    private static func createDefaultAttributes(item: Item) -> [CFString: Any] {
        return [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: keychainServiceName,
            kSecAttrAccount: item.rawValue,
        ]
    }

    private static func createAccessAttributes() -> [CFString: Any] {
        return [
            kSecAttrAccessGroup: ApplicationConfiguration.securityGroupIdentifier,
            kSecAttrAccessible: kSecAttrAccessibleAfterFirstUnlock,
        ]
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

        return Item(rawValue: accountNumber) == nil
    }
}

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
