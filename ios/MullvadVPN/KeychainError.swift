//
//  KeychainError.swift
//  MullvadVPN
//
//  Created by pronebird on 02/10/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Security

struct KeychainError: LocalizedError, Equatable {
    let code: OSStatus

    var errorDescription: String? {
        return SecCopyErrorMessageString(code, nil) as String?
    }

    static let duplicateItem = KeychainError(code: errSecDuplicateItem)
    static let itemNotFound = KeychainError(code: errSecItemNotFound)

    static func == (lhs: KeychainError, rhs: KeychainError) -> Bool {
        return lhs.code == rhs.code
    }
}
