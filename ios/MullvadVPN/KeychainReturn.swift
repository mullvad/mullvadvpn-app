//
//  KeychainReturn.swift
//  MullvadVPN
//
//  Created by pronebird on 24/04/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Security

extension Keychain {
    enum Return: KeychainAttributeEncodable, CaseIterable {
        case data
        case attributes
        case persistentReference

        fileprivate var attributeKey: CFString {
            switch self {
            case .attributes:
                return kSecReturnAttributes
            case .data:
                return kSecReturnData
            case .persistentReference:
                return kSecReturnPersistentRef
            }
        }

        func updateKeychainAttributes(in attributes: inout [CFString: Any]) {
            attributes[attributeKey] = true
        }
    }
}

extension Set: KeychainAttributeDecodable, KeychainAttributeEncodable
    where Element == Keychain.Return
{
    init?(attributes: [CFString: Any]) {
        let items = Keychain.Return.allCases.filter { (returnType) -> Bool in
            return attributes[returnType.attributeKey] as? Bool == .some(true)
        }

        if items.isEmpty {
            return nil
        } else {
            self.init(items)
        }
    }

    func updateKeychainAttributes(in attributes: inout [CFString : Any]) {
        Keychain.Return.allCases.forEach { (returnType) in
            attributes.removeValue(forKey: returnType.attributeKey)
        }

        forEach { $0.updateKeychainAttributes(in: &attributes) }
    }
}
