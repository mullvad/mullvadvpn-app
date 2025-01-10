//
//  KeychainSettingsStore.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-11-22.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import Security

public class KeychainSettingsStore: SettingsStore {
    public let serviceName: String
    public let accessGroup: String
    private let logger = Logger(label: "KeychainSettingsStore")
    private let cacheDirectory: URL

    public init(serviceName: String, accessGroup: String, cacheDirectory: URL) {
        self.serviceName = serviceName
        self.accessGroup = accessGroup
        self.cacheDirectory = cacheDirectory.appendingPathComponent("keychainLock.json")
    }

    public func read(key: SettingsKey) throws -> Data {
        try coordinate(Data(), try readItemData(key))
    }

    public func write(_ data: Data, for key: SettingsKey) throws {
        try coordinate((), try addOrUpdateItem(key, data: data))
    }

    public func delete(key: SettingsKey) throws {
        try coordinate((), try deleteItem(key))
    }

    /// Prevents all items in `keys` from backup inclusion
    ///
    /// This method uses the `coordinate` helper function to guarantee atomicity
    /// of the keychain exclusion process so that a pre-running VPN process cannot
    /// accidentally read or write to the keychain when the exclusion happens.
    /// It will be blocked temporarily and automatically resume when the migration is done.
    ///
    /// Likewise, the exclusion process will also be forced to wait until it can access the keychain
    /// if the VPN process is operating on it.
    ///
    /// - Important: Do not call `read`, `write`, or `delete` from this method,
    /// the coordinator is *not reentrant* and will deadlock if you do so.
    /// Only call methods that do not call `coordinate`.
    ///
    /// - Parameter keys: The keys to exclude from backup
    public func excludeFromBackup(keys: [SettingsKey]) {
        let coordination = { [unowned self] in
            for key in keys {
                do {
                    let data = try readItemData(key)
                    try deleteItem(key)
                    try addItem(key, data: data)
                } catch {
                    logger.error("Could not exclude \(key) from backups. \(error)")
                }
            }
        }

        try? coordinate((), coordination())
    }

    private func addItem(_ item: SettingsKey, data: Data) throws {
        var query = createDefaultAttributes(item: item)
        query.merge(createAccessAttributesThisDeviceOnly()) { current, _ in
            current
        }
        query[kSecValueData] = data

        let status = SecItemAdd(query as CFDictionary, nil)
        if status != errSecSuccess {
            throw KeychainError(code: status)
        }
    }

    private func updateItem(_ item: SettingsKey, data: Data) throws {
        let query = createDefaultAttributes(item: item)
        let status = SecItemUpdate(
            query as CFDictionary,
            [kSecValueData: data] as CFDictionary
        )

        if status != errSecSuccess {
            throw KeychainError(code: status)
        }
    }

    private func addOrUpdateItem(_ item: SettingsKey, data: Data) throws {
        do {
            try updateItem(item, data: data)
        } catch let error as KeychainError where error == .itemNotFound {
            try addItem(item, data: data)
        } catch {
            throw error
        }
    }

    private func readItemData(_ item: SettingsKey) throws -> Data {
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

    private func deleteItem(_ item: SettingsKey) throws {
        let query = createDefaultAttributes(item: item)
        let status = SecItemDelete(query as CFDictionary)
        if status != errSecSuccess {
            throw KeychainError(code: status)
        }
    }

    private func createDefaultAttributes(item: SettingsKey) -> [CFString: Any] {
        [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: serviceName,
            kSecAttrAccount: item.rawValue,
        ]
    }

    private func createAccessAttributesThisDeviceOnly() -> [CFString: Any] {
        [
            kSecAttrAccessGroup: accessGroup,
            kSecAttrAccessible: kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly,
        ]
    }

    /// Runs `action` in a cross process synchronized way
    ///
    /// This enables doing CRUD operations on the keychain items in a cross process safe way.
    /// This does not prevent TOCTOU issues.
    /// - Parameters:
    ///   - initial: Dummy value used for the returned value, if any.
    ///   - action: The CRUD operation to run on the keychain.
    /// - Returns: The result of the keychain operation, if any.
    private func coordinate<T>(_ initial: T, _ action: @autoclosure () throws -> T) throws -> T {
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)
        var error: NSError?
        var thrownError: Error?
        var returnedValue: T = initial
        fileCoordinator.coordinate(writingItemAt: cacheDirectory, error: &error) { _ in
            do {
                returnedValue = try action()
            } catch {
                thrownError = error
            }
        }

        if let thrownError {
            throw thrownError
        }

        return returnedValue
    }
}
