//
//  KeychainAttributes.swift
//  MullvadVPN
//
//  Created by pronebird on 22/04/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Security

extension Keychain {

    enum Accessible: RawRepresentable, CaseIterable, KeychainAttributeDecodable, KeychainAttributeEncodable {

        case whenPasscodeSetThisDeviceOnly
        case whenUnlocked
        case whenUnlockedThisDeviceOnly
        case afterFirstUnlock
        case afterFirstUnlockThisDeviceOnly

        var rawValue: CFString {
            switch self {
            case .whenPasscodeSetThisDeviceOnly:
                return kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly
            case .whenUnlocked:
                return kSecAttrAccessibleWhenUnlocked
            case .whenUnlockedThisDeviceOnly:
                return kSecAttrAccessibleWhenUnlockedThisDeviceOnly
            case .afterFirstUnlock:
                return kSecAttrAccessibleAfterFirstUnlock
            case .afterFirstUnlockThisDeviceOnly:
                return kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
            }
        }

        init?(rawValue: CFString) {
            let maybeCase = Self.allCases.first { $0.rawValue == rawValue }

            if let maybeCase = maybeCase {
                self = maybeCase
            } else {
                return nil
            }
        }

        init?(attributes: [CFString: Any]) {
            if let rawValue = attributes[kSecAttrAccessible] as? String {
                self.init(rawValue: rawValue as CFString)
            } else {
                return nil
            }
        }

        func updateKeychainAttributes(in attributes: inout [CFString : Any]) {
            attributes[kSecAttrAccessible] = rawValue
        }

    }

    struct Attributes: KeychainAttributeEncodable, KeychainAttributeDecodable {
        var `class`: KeychainClass?
        var service: String?
        var account: String?
        var accessGroup: String?
        var accessible: Accessible?
        var creationDate: Date?
        var modificationDate: Date?
        var generic: Data?

        var valueData: Data?
        var valuePersistentReference: Data?

        var `return`: Set<Keychain.Return>?
        var matchLimit: Keychain.MatchLimit?

        init() {}

        init(attributes: [CFString: Any]) {
            `class` = KeychainClass(attributes: attributes)
            service = attributes[kSecAttrService] as? String
            account = attributes[kSecAttrAccount] as? String
            accessGroup = attributes[kSecAttrAccessGroup] as? String
            accessible = Accessible(attributes: attributes)
            creationDate = attributes[kSecAttrCreationDate] as? Date
            modificationDate = attributes[kSecAttrModificationDate] as? Date
            generic = attributes[kSecAttrGeneric] as? Data

            valueData = attributes[kSecValueData] as? Data
            valuePersistentReference = attributes[kSecValuePersistentRef] as? Data

            `return` = Set(attributes: attributes)
            matchLimit = Keychain.MatchLimit(attributes: attributes)
        }

        func updateKeychainAttributes(in attributes: inout [CFString: Any]) {
            `class`?.updateKeychainAttributes(in: &attributes)

            if let service = service {
                attributes[kSecAttrService] = service
            }

            if let account = account {
                attributes[kSecAttrAccount] = account
            }

            if let accessGroup = accessGroup {
                attributes[kSecAttrAccessGroup] = accessGroup
            }

            accessible?.updateKeychainAttributes(in: &attributes)

            if let creationDate = creationDate {
                attributes[kSecAttrCreationDate] = creationDate
            }

            if let modificationDate = modificationDate {
                attributes[kSecAttrModificationDate] = modificationDate
            }

            if let generic = generic {
                attributes[kSecAttrGeneric] = generic
            }

            if let valueData = valueData {
                attributes[kSecValueData] = valueData
            }

            if let valuePersistentReference = valuePersistentReference {
                attributes[kSecValuePersistentRef] = valuePersistentReference
            }

            `return`?.updateKeychainAttributes(in: &attributes)
            matchLimit?.updateKeychainAttributes(in: &attributes)
        }

    }

}
