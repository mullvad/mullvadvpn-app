//
//  SettingsManager.swift
//  MullvadVPN
//
//  Created by pronebird on 29/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

enum SettingsManager {}

struct LegacyTunnelSettings {
    let accountNumber: String
    let tunnelSettings: TunnelSettingsV1
}

let keychainServiceName = "Mullvad VPN"

enum KeychainAccountName: String, CaseIterable {
    case settings = "Settings"
    case lastUsedAccount = "LastUsedAccount"
    case pinnedLocationNames = "PinnedLocationNames"
}

extension SettingsManager {

    // MARK: -

    static func getLastUsedAccount() throws -> String {
        var query = createDefaultAttributes(accountName: .lastUsedAccount)
        query[kSecReturnData] = true

        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        guard status == errSecSuccess else {
            throw KeychainError(code: status)
        }

        let data = result as! Data

        return String(data: data, encoding: .utf8)!
    }

    static func setLastUsedAccount(_ string: String?) throws {
        let query = createDefaultAttributes(accountName: .lastUsedAccount)

        guard let string = string else {
            switch SecItemDelete(query as CFDictionary) {
            case errSecSuccess, errSecItemNotFound:
                return
            case let status:
                throw KeychainError(code: status)
            }
        }

        let data = string.data(using: .utf8)!
        var status = SecItemUpdate(
            query as CFDictionary,
            [kSecValueData: data] as CFDictionary
        )

        switch status {
        case errSecItemNotFound:
            var insert = query
            insert[kSecAttrAccessible] = kSecAttrAccessibleAfterFirstUnlock
            insert[kSecValueData] = data

            status = SecItemAdd(insert as CFDictionary, nil)
            if status != errSecSuccess {
                throw KeychainError(code: status)
            }
        case errSecSuccess:
            break
        default:
            throw KeychainError(code: status)
        }
    }

    // MARK: -

    static func readSettings() throws -> TunnelSettingsV2 {
        var query = createDefaultAttributes(accountName: .settings)
        query[kSecReturnData] = true

        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        guard status == errSecSuccess else {
            throw KeychainError(code: status)
        }

        let data = result as! Data

        let decoder = JSONDecoder()
        return try decoder.decode(TunnelSettingsV2.self, from: data)
    }

    static func writeSettings(_ settings: TunnelSettingsV2) throws {
        let encoder = JSONEncoder()
        let data = try encoder.encode(settings)

        let query = createDefaultAttributes(accountName: .settings)
        var status = SecItemUpdate(
            query as CFDictionary,
            [kSecValueData: data] as CFDictionary
        )

        switch status {
        case errSecItemNotFound:
            var insert = query
            insert[kSecAttrAccessGroup] = ApplicationConfiguration.securityGroupIdentifier
            insert[kSecAttrAccessible] = kSecAttrAccessibleAfterFirstUnlock
            insert[kSecValueData] = data

            status = SecItemAdd(insert as CFDictionary, nil)
            if status != errSecSuccess {
                throw KeychainError(code: status)
            }
        case errSecSuccess:
            break
        default:
            throw KeychainError(code: status)
        }
    }

    static func deleteSettings() throws {
        let query = createDefaultAttributes(accountName: .settings)
        let status = SecItemDelete(query as CFDictionary)
        if status != errSecSuccess {
            throw KeychainError(code: status)
        }
    }

    private static func createDefaultAttributes(accountName: KeychainAccountName) -> [CFString: Any]  {
        return [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: keychainServiceName,
            kSecAttrAccount: accountName.rawValue
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
            kSecMatchLimit: kSecMatchLimitAll
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
                      let data = item[kSecValueData] as? Data else {
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
                        chainedError: AnyChainedError(error),
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
            kSecMatchLimit: kSecMatchLimitAll
        ]

        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        guard status == errSecSuccess else {
            let error = KeychainError(code: status)

            if error != .itemNotFound {
                logger.error(
                    chainedError: AnyChainedError(error),
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
            .forEach { (index, item) in
                guard let account = item[kSecAttrAccount] else {
                    return
                }

                let deleteQuery: [CFString: Any] = [
                    kSecClass: kSecClassGenericPassword,
                    kSecAttrService: keychainServiceName,
                    kSecAttrAccount: account
                ]

                let status = SecItemDelete(deleteQuery as CFDictionary)
                if status == errSecSuccess {
                    logger.debug("Removed legacy settings entry \(index).")
                } else {
                    let error = KeychainError(code: status)

                    logger.error(
                        chainedError: AnyChainedError(error),
                        message: "Failed to remove legacy settings entry \(index)."
                    )
                }
            }
    }

    private static func filterLegacySettings(_ item: [CFString: Any]) -> Bool {
        guard let accountNumber = item[kSecAttrAccount] as? String else {
            return false
        }

        return KeychainAccountName(rawValue: accountNumber) == nil
    }
}

extension SettingsManager {
    
    // MARK: - Pinned location names

    static func getPinnedLocationNames() throws -> Set<String> {
        var query = createDefaultAttributes(accountName: .pinnedLocationNames)
        query[kSecReturnData] = true

        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        guard status == errSecSuccess else {
            throw KeychainError(code: status)
        }

        guard let data = result as? Data else {
            throw KeychainError.itemNotFound
        }
        
        guard let displayNames = try NSKeyedUnarchiver.unarchivedObject(
            ofClass: NSSet.self,
            from: data
        ) as? Set<String> else {
            throw KeychainError.itemNotFound
        }
        
        return displayNames
    }

    static func setPinnedLocationNames(_ displayNames: Set<String>?) throws {
        let query = createDefaultAttributes(accountName: .pinnedLocationNames)

        guard let displayNames = displayNames else {
            switch SecItemDelete(query as CFDictionary) {
            case errSecSuccess, errSecItemNotFound:
                return
            case let status:
                throw KeychainError(code: status)
            }
        }

        let data = try NSKeyedArchiver.archivedData(
            withRootObject: displayNames,
            requiringSecureCoding: true
        )
        var status = SecItemUpdate(
            query as CFDictionary,
            [kSecValueData: data] as CFDictionary
        )

        switch status {
        case errSecItemNotFound:
            var insert = query
            insert[kSecAttrAccessible] = kSecAttrAccessibleAfterFirstUnlock
            insert[kSecValueData] = data

            status = SecItemAdd(insert as CFDictionary, nil)
            if status != errSecSuccess {
                throw KeychainError(code: status)
            }
        case errSecSuccess:
            break
        default:
            throw KeychainError(code: status)
        }
    }
}
