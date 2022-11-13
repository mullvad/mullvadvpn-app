//
//  KeychainSettingsStore.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-11-22.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Security

class KeychainSettingsStore: SettingsStore {
    let keychainServiceName: String

    init(keychainServiceName: String) {
        self.keychainServiceName = keychainServiceName
    }

    func read(key: SettingsKey) throws -> Data {
        try readItemData(key)
    }

    func write(_ data: Data, for key: SettingsKey) throws {
        try addOrUpdateItem(key, data: data)
    }

    func delete(key: SettingsKey) throws {
        try deleteItem(key)
    }

    private func addItem(_ item: SettingsKey, data: Data) throws {
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
