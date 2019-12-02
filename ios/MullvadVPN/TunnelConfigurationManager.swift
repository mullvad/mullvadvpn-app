//
//  TunnelConfigurationManager.swift
//  MullvadVPN
//
//  Created by pronebird on 02/10/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import Security

/// Service name used for keychain items
private let kServiceName = "Mullvad VPN"

enum TunnelConfigurationManagerError: Error {
    case encode(TunnelConfigurationCoder.Error)
    case decode(TunnelConfigurationCoder.Error)
    case addToKeychain(KeychainError)
    case updateKeychain(KeychainError)
    case removeKeychainItem(KeychainError)
    case getFromKeychain(KeychainError)
    case getPersistentKeychainRef(KeychainError)
}

enum TunnelConfigurationManager {}

extension TunnelConfigurationManager {

    static func save(configuration: TunnelConfiguration, account: String) -> Result<(), TunnelConfigurationManagerError> {
        TunnelConfigurationCoder.encode(tunnelConfig: configuration)
            .mapError { .encode($0) }
            .flatMap { (data) -> Result<(), TunnelConfigurationManagerError> in
                Keychain.updateItem(account: account, data: data)
                    .flatMapError { (keychainError) -> Result<(), TunnelConfigurationManagerError> in
                        if case .itemNotFound = keychainError {
                            return Keychain.addItem(account: account, data: data)
                                .mapError { .addToKeychain($0) }
                        } else {
                            return .failure(.updateKeychain(keychainError))
                        }
                }
        }
    }

    static func load(account: String) -> Result<TunnelConfiguration, TunnelConfigurationManagerError> {
        Keychain.getItemData(account: account)
            .mapError { .getFromKeychain($0) }
            .flatMap { (data) in
                TunnelConfigurationCoder.decode(data: data)
                    .mapError { .decode($0) }
        }
    }

    static func load(persistentKeychainRef: Data) -> Result<TunnelConfiguration, TunnelConfigurationManagerError> {
        Keychain.getItemData(persistentKeychainRef: persistentKeychainRef)
            .mapError { .getFromKeychain($0) }
            .flatMap { (data) in
                TunnelConfigurationCoder.decode(data: data)
                    .mapError { .decode($0) }
        }
    }

    static func remove(account: String) -> Result<(), TunnelConfigurationManagerError> {
        Keychain.removeItem(account: account)
            .mapError { .removeKeychainItem($0) }
    }

    static func getPersistentKeychainRef(account: String) -> Result<Data, TunnelConfigurationManagerError> {
        Keychain.getPersistentRef(account: account)
            .mapError { .getPersistentKeychainRef($0) }
    }

}

private enum Keychain {}

private extension Keychain {

    static func listAccounts() -> Result<[String], KeychainError> {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: kServiceName,
            kSecReturnAttributes: true,
            kSecMatchLimit: kSecMatchLimitAll,
        ]

        var ref: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &ref)

        if status == errSecSuccess {
            let attrs = ref as! [[CFString: Any]]
            let accountTokens = attrs.compactMap { dict in dict[kSecAttrAccount] as? String }

            return .success(accountTokens)
        } else {
            return .failure(KeychainError(code: status))
        }
    }

    static func getPersistentRef(account: String) -> Result<Data, KeychainError> {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: account,
            kSecAttrService: kServiceName,
            kSecReturnPersistentRef: true
        ]

        var ref: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &ref)

        if status == errSecSuccess {
            return .success(ref as! Data)
        } else {
            return .failure(KeychainError(code: status))
        }
    }

    static func getItemData(persistentKeychainRef: Data) -> Result<Data, KeychainError> {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecValuePersistentRef: persistentKeychainRef,
            kSecReturnData: true
        ]

        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        if status == errSecSuccess {
            return .success(result as! Data)
        } else {
            return .failure(KeychainError(code: status))
        }
    }

    static func getItemData(account: String) -> Result<Data, KeychainError> {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: account,
            kSecAttrService: kServiceName,
            kSecReturnData: true
        ]

        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        if status == errSecSuccess {
            return .success(result as! Data)
        } else {
            return .failure(KeychainError(code: status))
        }
    }

    static func addItem(account: String, data: Data) -> Result<(), KeychainError> {
        let attributes: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: account,
            kSecAttrService: kServiceName,
            kSecValueData: data,
            kSecReturnData: false,

            // Share the key with the application group
            kSecAttrAccessGroup: ApplicationConfiguration.securityGroupIdentifier,
        ]

        var ref: CFTypeRef?
        let status = SecItemAdd(attributes as CFDictionary, &ref)

        if status == errSecSuccess {
            return .success(())
        } else {
            return .failure(KeychainError(code: status))
        }
    }

    static func updateItem(account: String, data: Data) -> Result<(), KeychainError> {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: account,
            kSecAttrService: kServiceName,
        ]

        let update: [CFString: Any] = [
            kSecValueData: data
        ]

        let status = SecItemUpdate(query as CFDictionary, update as CFDictionary)

        if status == errSecSuccess {
            return .success(())
        } else {
            return .failure(KeychainError(code: status))
        }
    }

    static func removeItem(account: String) -> Result<(), KeychainError> {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: account,
            kSecAttrService: kServiceName,
        ]

        let status = SecItemDelete(query as CFDictionary)

        if status == errSecSuccess {
            return .success(())
        } else {
            return .failure(KeychainError(code: status))
        }
    }

}
