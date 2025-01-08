//
//  KeychainSettingsStoreMigration.swift
//  MullvadSettings
//
//  Created by Marco Nikic on 2025-01-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct KeychainSettingsStoreMigration {
    private let serviceName: String
    private let accessGroup: String
    private let store: SettingsStore

    init(serviceName: String, accessGroup: String, store: SettingsStore) {
        self.serviceName = serviceName
        self.accessGroup = accessGroup
        self.store = store
    }

    /// Creates a keychain query matching old keychain settings that would be included in backups.
    private func createQueryAttributes(item: SettingsKey) -> [CFString: Any] {
        [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: serviceName,
            kSecAttrAccount: item.rawValue,
            kSecAttrAccessGroup: accessGroup,
            kSecAttrAccessible: kSecAttrAccessibleAfterFirstUnlock,
        ]
    }

    /// Whether items saved in the keychain should be migrated to be excluded from backups.
    ///
    /// Create a keychain item query using the old value `kSecAttrAccessibleAfterFirstUnlock`
    /// for keychain items accessibility.
    ///
    /// If there is a match, probe whether it's a first time launch by querying whether the `SettingKey`
    /// `shouldWipeSettings` has already been set.
    ///
    /// If both the query is successful, and `shouldWipeSettings` has already been set,
    /// this means the keychain saved settings accessibility need to be upgraded to `kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly`.
    ///
    /// This will be done automatically by deleting and re-adding entries in the keychain.
    /// - Returns: Whether keychain settings should be deleted and added again
    private func shouldExcludeSettingsFromBackup() -> Bool {
        var query = createQueryAttributes(item: .shouldWipeSettings)
        query[kSecReturnData] = true

        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        let shouldWipeSettingsWasSet = (try? store.read(key: .shouldWipeSettings)) != nil

        return status == errSecSuccess && shouldWipeSettingsWasSet
    }

    public func excludeKeychainSettingsFromBackups() {
        guard shouldExcludeSettingsFromBackup() == true else { return }
        store.excludeFromBackup(keys: SettingsKey.allCases)

        precondition(shouldExcludeSettingsFromBackup() == false)
    }
}
