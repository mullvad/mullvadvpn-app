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

    /// A Keychain Result type
    typealias Result<T> = Swift.Result<T, KeychainError>

    /// List all of the account tokens in Keychain
    static func listAccounts() -> Result<[String]> {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: kServiceName,
            kSecReturnAttributes: true,
            kSecMatchLimit: kSecMatchLimitAll,
        ]

        return executeSecCopyMatching(query: query)
            .map { (result) in
                let attrs = result as! [[CFString: Any]]
                let accountTokens = attrs.compactMap { (dict) in
                    dict[kSecAttrAccount] as? String
                }

                return accountTokens
        }
    }

    /// Get a persistent reference to the Keychain item for the given account token
    static func getPersistentRef(account: String) -> Result<Data> {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: account,
            kSecAttrService: kServiceName,
            kSecReturnPersistentRef: true
        ]

        return executeSecCopyMatching(query: query)
            .map { $0 as! Data }
    }

    /// Get data associated with the given persistent Keychain reference
    static func getItemData(persistentKeychainRef: Data) -> Result<Data> {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecValuePersistentRef: persistentKeychainRef,
            kSecReturnData: true
        ]

        return executeSecCopyMatching(query: query)
            .map { $0 as! Data }
    }

    /// Get data associated with the given account token
    static func getItemData(account: String) -> Result<Data> {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: account,
            kSecAttrService: kServiceName,
            kSecReturnData: true
        ]

        return executeSecCopyMatching(query: query)
            .map { $0 as! Data }
    }

    /// Store data in the Keychain and associate it with the given account token
    static func addItem(account: String, data: Data) -> Result<()> {
        let attributes: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: account,
            kSecAttrService: kServiceName,
            kSecValueData: data,
            kSecReturnData: false,

            // Share the item with the application group
            kSecAttrAccessGroup: ApplicationConfiguration.securityGroupIdentifier,
        ]

        let status = SecItemAdd(attributes as CFDictionary, nil)

        return mapSecResult(status: status) {
            ()
        }
    }

    /// Replace the data associated with the given account token.
    static func updateItem(account: String, data: Data) -> Result<()> {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: account,
            kSecAttrService: kServiceName,
        ]

        let update: [CFString: Any] = [
            kSecValueData: data
        ]

        let status = SecItemUpdate(query as CFDictionary, update as CFDictionary)

        return mapSecResult(status: status) {
            ()
        }
    }

    /// Remove the data associated with the given account token
    static func removeItem(account: String) -> Result<()> {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: account,
            kSecAttrService: kServiceName,
        ]

        let status = SecItemDelete(query as CFDictionary)

        return mapSecResult(status: status) {
            ()
        }
    }

    /// A private helper that verifies the given `status` and executes `body` on success
    static private func mapSecResult<T>(status: OSStatus, body: () -> T) -> Result<T> {
        if status == errSecSuccess {
            return .success(body())
        } else {
            return .failure(KeychainError(code: status))
        }
    }

    /// A private helper to execute the given query using `SecCopyMatching` and map the result to
    /// the `Result<CFTypeRef?>` type.
    static private func executeSecCopyMatching(query: [CFString: Any]) -> Result<CFTypeRef?> {
        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        return mapSecResult(status: status) {
            result
        }
    }

}
