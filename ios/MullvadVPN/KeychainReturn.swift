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
    enum Return: KeychainAttributeEncodable {
        case data
        case attributes
        case persistentReference

        func updateKeychainAttributes(in attributes: inout [CFString: Any]) {
            switch self {
            case .attributes:
                attributes[kSecReturnAttributes] = true
            case .data:
                attributes[kSecReturnData] = true
            case .persistentReference:
                attributes[kSecReturnPersistentRef] = true
            }
        }
    }
}

extension Set: KeychainAttributeDecodable, KeychainAttributeEncodable where Element == Keychain.Return {
    init?(attributes: [CFString: Any]) {
        var items = [Keychain.Return]()

        if let value = attributes[kSecReturnAttributes] as? Bool, value {
            items.append(.attributes)
        }

        if let value = attributes[kSecReturnData] as? Bool, value {
            items.append(.data)
        }

        if let value = attributes[kSecReturnPersistentRef] as? Bool, value {
            items.append(.persistentReference)
        }

        if items.isEmpty {
            return nil
        } else {
            self.init(items)
        }
    }

    func updateKeychainAttributes(in attributes: inout [CFString : Any]) {
        forEach { $0.updateKeychainAttributes(in: &attributes) }
    }
}
