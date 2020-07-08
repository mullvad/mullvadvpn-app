//
//  KeychainError.swift
//  MullvadVPN
//
//  Created by pronebird on 02/10/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Security

extension Keychain {
    struct Error: Swift.Error, LocalizedError {
        let code: OSStatus

        var errorDescription: String? {
            return SecCopyErrorMessageString(code, nil) as String?
        }
    }
}


extension Keychain.Error {

    static let duplicateItem = Keychain.Error(code: errSecDuplicateItem)
    static let itemNotFound = Keychain.Error(code: errSecItemNotFound)

    static func ~= (lhs: Keychain.Error, rhs: Swift.Error) -> Bool {
        guard let rhsError = rhs as? Keychain.Error else { return false }
        return lhs.code == rhsError.code
    }
}
