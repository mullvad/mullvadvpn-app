//
//  KeychainError.swift
//  MullvadVPN
//
//  Created by pronebird on 02/10/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Security

public struct KeychainError: LocalizedError, Equatable {
    public let code: OSStatus
    public init(code: OSStatus) {
        self.code = code
    }

    public var errorDescription: String? {
        return SecCopyErrorMessageString(code, nil) as String?
    }

    public static let duplicateItem = KeychainError(code: errSecDuplicateItem)
    public static let itemNotFound = KeychainError(code: errSecItemNotFound)
    public static let interactionNotAllowed = KeychainError(code: errSecInteractionNotAllowed)

    public static func == (lhs: KeychainError, rhs: KeychainError) -> Bool {
        return lhs.code == rhs.code
    }
}
