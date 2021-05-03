//
//  TunnelSettingsManager.swift
//  MullvadVPN
//
//  Created by pronebird on 02/10/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Security

/// Service name used for keychain items
private let kServiceName = "Mullvad VPN"

/// Maximum number of attempts to perform when updating the Keychain entry "atomically"
private let kMaxAtomicUpdateRetryLimit = 20

enum TunnelSettingsManager {}

extension TunnelSettingsManager {

    enum Error: ChainedError {
        /// A failure to encode the given tunnel settings
        case encode(Swift.Error)

        /// A failure to decode the data stored in Keychain
        case decode(Swift.Error)

        /// A failure to add a new entry to Keychain
        case addEntry(Keychain.Error)

        /// A failure to update the existing entry in Keychain
        case updateEntry(Keychain.Error)

        /// A failure to atomically update a Keychain entry after multiple attempts
        case updateEntryAtomicallyRetryLimitExceeded

        /// A failure to remove an entry in Keychain
        case removeEntry(Keychain.Error)

        /// A failure to query the entry in Keychain
        case lookupEntry(Keychain.Error)
    }

    typealias Result<T> = Swift.Result<T, Error>

    /// Keychain access level that should be used for all items containing tunnel settings
    private static let keychainAccessibleLevel = Keychain.Accessible.afterFirstUnlock

    enum KeychainSearchTerm {
        case accountToken(String)
        case persistentReference(Data)

        /// Returns `Keychain.Attributes` appropriate for adding or querying the item
        fileprivate func makeKeychainAttributes() -> Keychain.Attributes {
            var attributes = Keychain.Attributes()
            attributes.class = .genericPassword

            switch self {
            case .accountToken(let accountToken):
                attributes.account = accountToken
                attributes.service = kServiceName

            case .persistentReference(let persistentReference):
                attributes.valuePersistentReference = persistentReference
            }

            return attributes
        }
    }

    struct KeychainEntry {
        let accountToken: String
        let tunnelSettings: TunnelSettings
    }

    static func load(searchTerm: KeychainSearchTerm) -> Result<KeychainEntry> {
        var query = searchTerm.makeKeychainAttributes()
        query.return = [.data, .attributes]

        return Keychain.findFirst(query: query)
            .mapError { .lookupEntry($0) }
            .flatMap { (attributes) in
                let attributes = attributes!
                let account = attributes.account!
                let data = attributes.valueData!

                return Self.decode(data: data)
                    .map { KeychainEntry(accountToken: account, tunnelSettings: $0) }
        }
    }

    static func add(configuration: TunnelSettings, account: String) -> Result<()> {
        Self.encode(tunnelConfig: configuration)
            .flatMap { (data) -> Result<()> in
                var attributes = KeychainSearchTerm.accountToken(account)
                    .makeKeychainAttributes()

                // Share the item with the application group
                attributes.accessGroup = ApplicationConfiguration.securityGroupIdentifier

                // Make sure the keychain item is available after the first unlock to enable
                // automatic key rotation in background (from the packet tunnel process)
                attributes.accessible = Self.keychainAccessibleLevel

                // Store value
                attributes.valueData = data

                // Add revision
                KeychainItemRevision.firstRevision().store(in: &attributes)

                return Keychain.add(attributes)
                    .mapError { .addEntry($0) }
                    .map { _ in () }
        }
    }

    /// This is a migration path for the existing Keychain entries created by 2020.2 or before.
    ///
    /// - Set the appropriate `accessible` so that the Packet Tunnel can access the tunnel
    ///    configuration when the device is locked.
    /// - Add revision field
    ///
    /// - Returns: A boolean that indicates whether the entry was up to date prior to the
    ///            migration request.

    static func migrateKeychainEntry(searchTerm: KeychainSearchTerm) -> Result<Bool> {
        var queryAttributes = searchTerm.makeKeychainAttributes()
        queryAttributes.return = [.attributes]

        return Keychain.findFirst(query: queryAttributes)
            .mapError { .lookupEntry($0) }
            .flatMap { (itemAttributes) -> Result<Bool> in
                let itemAttributes = itemAttributes!

                let searchAttributes = searchTerm.makeKeychainAttributes()
                var updateAttributes = Keychain.Attributes()

                // Add revision if it's missing
                if KeychainItemRevision(attributes: itemAttributes) == nil {
                    KeychainItemRevision.firstRevision().store(in: &updateAttributes)
                }

                // Fix the accessibility permission for the Keychain entry
                if itemAttributes.accessible != Self.keychainAccessibleLevel {
                    updateAttributes.accessible = Self.keychainAccessibleLevel
                }

                // Return immediately if nothing to update (i.e the keychain query is empty)
                if updateAttributes.keychainRepresentation().isEmpty {
                    return .success(false)
                } else {
                    return Keychain.update(query: searchAttributes, update: updateAttributes)
                        .mapError { .updateEntry($0) }
                        .map { true }
                }
        }
    }

    /// Reads the tunnel settings from Keychain, then passes it to the given closure for
    /// modifications, saves the result back to Keychain.
    ///
    /// The given block may run multiple times if Keychain entry was changed between read and write
    /// operations.
    static func update(searchTerm: KeychainSearchTerm,
                       using changeConfiguration: (inout TunnelSettings) -> Void)
        -> Result<TunnelSettings>
    {
        for _ in (0 ..< kMaxAtomicUpdateRetryLimit) {
            var searchQuery = searchTerm.makeKeychainAttributes()
            searchQuery.return = [.attributes, .data]

            let result = Keychain.findFirst(query: searchQuery)
                .mapError { .lookupEntry($0) }
                .flatMap { (itemAttributes) -> Result<TunnelSettings> in
                    let itemAttributes = itemAttributes!
                    let serializedData = itemAttributes.valueData!
                    let account = itemAttributes.account!

                    // Parse the current revision from Keychain attributes
                    let currentRevision = KeychainItemRevision(attributes: itemAttributes)

                    // Pick the next revision in sequence
                    let nextRevision = currentRevision?.nextRevision
                        ?? KeychainItemRevision.firstRevision()

                    return Self.decode(data: serializedData)
                        .flatMap { (tunnelConfig) -> Result<TunnelSettings> in
                            var tunnelConfig = tunnelConfig
                            changeConfiguration(&tunnelConfig)

                            return Self.encode(tunnelConfig: tunnelConfig)
                                .flatMap { (newData) -> Result<TunnelSettings> in
                                    // `SecItemUpdate` does not accept query parameters when using
                                    // persistent reference, so constraint the query to account
                                    // token instead now when we know it
                                    var updateQuery = KeychainSearchTerm
                                        .accountToken(account)
                                        .makeKeychainAttributes()

                                    // Provide the last known revision via generic field to prevent
                                    // overwriting the item if it was modified in the meanwhile.
                                    // This field can be missing for the existing apps on AppStore
                                    currentRevision?.store(in: &updateQuery)

                                    var updateAttributes = Keychain.Attributes()
                                    updateAttributes.valueData = newData

                                    // Add the next revision number
                                    nextRevision.store(in: &updateAttributes)

                                    return Keychain.update(query: updateQuery, update: updateAttributes)
                                        .mapError { .updateEntry($0) }
                                        .map { tunnelConfig }
                            }
                        }
            }

            // Retry if Keychain reported that the item was not found when updating
            if case .failure(.updateEntry(.itemNotFound)) = result  {
                continue
            } else {
                return result
            }
        }

        return .failure(.updateEntryAtomicallyRetryLimitExceeded)
    }

    static func remove(searchTerm: KeychainSearchTerm) -> Result<()> {
        return Keychain.delete(query: searchTerm.makeKeychainAttributes())
            .mapError { .removeEntry($0) }
    }

    /// Get a persistent reference to the Keychain item for the given account token
    static func getPersistentKeychainReference(account: String) -> Result<Data> {
        var query = KeychainSearchTerm.accountToken(account)
            .makeKeychainAttributes()
        query.return = [.persistentReference]

        return Keychain.findFirst(query: query)
            .mapError { .lookupEntry($0) }
            .map { (attributes) -> Data in
                return attributes!.valuePersistentReference!
        }
    }

    /// Verify that the keychain entry exists.
    /// Returns an error in case of failure to access Keychain.
    static func exists(searchTerm: KeychainSearchTerm) -> Result<Bool> {
        let query = searchTerm.makeKeychainAttributes()

        return Keychain.findFirst(query: query)
            .map({ (attributes) -> Bool in
                return true
            })
            .flatMapError({ (error) -> Result<Bool> in
                if case .itemNotFound = error {
                    return .success(false)
                } else {
                    return .failure(.lookupEntry(error))
                }
            })
    }

    private static func encode(tunnelConfig: TunnelSettings) -> Result<Data> {
        return Swift.Result { try JSONEncoder().encode(tunnelConfig) }
            .mapError { .encode($0) }
    }

    private static func decode(data: Data) -> Result<TunnelSettings> {
        return Swift.Result { try JSONDecoder().decode(TunnelSettings.self, from: data) }
            .mapError { .decode($0) }
    }
}
