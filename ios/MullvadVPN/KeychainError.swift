//
//  KeychainError.swift
//  MullvadVPN
//
//  Created by pronebird on 02/10/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import Security

struct KeychainError: Error, LocalizedError {
    let code: OSStatus

    var errorDescription: String? {
        return SecCopyErrorMessageString(code, nil) as String?
    }
}

extension KeychainError {

    static let duplicateItem = KeychainError(code: errSecDuplicateItem)
    static let itemNotFound = KeychainError(code: errSecItemNotFound)

    static func ~= (lhs: KeychainError, rhs: Error) -> Bool {
        guard let rhsError = rhs as? KeychainError else { return false }
        return lhs.code == rhsError.code
    }
}
