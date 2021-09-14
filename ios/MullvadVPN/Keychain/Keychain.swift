//
//  Keychain.swift
//  MullvadVPN
//
//  Created by pronebird on 22/04/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Security

protocol KeychainAttributeDecodable {
    init?(attributes: [CFString: Any])
}

protocol KeychainAttributeEncodable {
    func keychainRepresentation() -> [CFString: Any]
    func updateKeychainAttributes(in attributes: inout [CFString: Any])
}

extension KeychainAttributeEncodable {
    func keychainRepresentation() -> [CFString: Any] {
        var attributes = [CFString: Any]()
        updateKeychainAttributes(in: &attributes)
        return attributes
    }
}

enum Keychain {}

extension Keychain {

    /// A Keychain Result type
    typealias Result<T> = Swift.Result<T, Keychain.Error>

    static func add(_ attributes: Keychain.Attributes) -> Result<Keychain.Attributes?> {
        var result: CFTypeRef?
        let status = SecItemAdd(attributes.keychainRepresentation() as CFDictionary, &result)

        return mapSecResultAndReturnValue(
            status: status,
            value: result,
            returnSet: attributes.return ?? [],
            limit: .one)
            .map { $0.first }
    }

    static func update(query: Keychain.Attributes, update: Keychain.Attributes) -> Result<()> {
        let queryAttributes = query.keychainRepresentation() as CFDictionary
        let updateAttributes = update.keychainRepresentation() as CFDictionary

        let status = SecItemUpdate(queryAttributes, updateAttributes)

        return mapSecResult(status: status) {
            return ()
        }
    }

    static func delete(query: Keychain.Attributes) -> Result<()> {
        let status = SecItemDelete(query.keychainRepresentation() as CFDictionary)

        return mapSecResult(status: status) {
            return ()
        }
    }

    static func findFirst(query: Keychain.Attributes) -> Result<Keychain.Attributes?> {
        return find(query: query).map { $0.first }
    }

    static func find(query: Keychain.Attributes) -> Result<[Keychain.Attributes]> {
        let attributes = query.keychainRepresentation()

        var result: CFTypeRef?
        let status = SecItemCopyMatching(attributes as CFDictionary, &result)

        return mapSecResultAndReturnValue(
            status: status,
            value: result,
            returnSet: query.return ?? [],
            limit: query.matchLimit ?? .one
        )
    }

    static private func mapSecResultAndReturnValue(
        status: OSStatus,
        value: CFTypeRef?,
        returnSet: Set<Keychain.Return>,
        limit: Keychain.MatchLimit) -> Result<[Keychain.Attributes]>
    {
        return mapSecResult(status: status) { () -> [Keychain.Attributes] in
            return value.map { parseReturnValue(value: $0, returnSet: returnSet, limit: limit) }
                ?? []
        }
    }

    static private func parseReturnValue(
        value: CFTypeRef,
        returnSet: Set<Keychain.Return>,
        limit: Keychain.MatchLimit) -> [Keychain.Attributes]
    {
        switch returnSet {
        case []:
            return []

        case [.data]:
            let values: [Data] = unsafelyCastReturnValue(value: value, limit: limit)

            return values.map { (data) -> Keychain.Attributes in
                var attributes = Keychain.Attributes()
                attributes.valueData = data
                return attributes
            }

        case [.persistentReference]:
            let values: [Data] = unsafelyCastReturnValue(value: value, limit: limit)

            return values.map { (persistentReference) -> Keychain.Attributes in
                var attributes = Keychain.Attributes()
                attributes.valuePersistentReference = persistentReference
                return attributes
            }

        default:
            let rawAttributeList: [[CFString: Any]] =
                unsafelyCastReturnValue(value: value, limit: limit)

            return rawAttributeList.map { Keychain.Attributes(attributes: $0) }
        }
    }

    /// A private helper that casts and normalizes the return value from Keychain to produce
    /// an array even when a single item is expected to be returned.
    static private func unsafelyCastReturnValue<T>(
        value: CFTypeRef,
        limit: Keychain.MatchLimit) -> [T]
    {
        switch limit {
        case .one:
            return [value as! T]
        case .all:
            return value as! [T]
        }
    }

    /// A private helper that verifies the given `status` and executes `body` on success
    static private func mapSecResult<T>(status: OSStatus, body: () -> T) -> Result<T> {
        if status == errSecSuccess {
            return .success(body())
        } else {
            return .failure(Keychain.Error(code: status))
        }
    }
}
