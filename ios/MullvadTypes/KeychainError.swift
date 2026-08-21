// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import Security

public struct KeychainError: LocalizedError, Equatable {
    public let code: OSStatus
    public init(code: OSStatus) {
        self.code = code
    }

    public var errorDescription: String? {
        SecCopyErrorMessageString(code, nil) as String?
    }

    public static let duplicateItem = KeychainError(code: errSecDuplicateItem)
    public static let itemNotFound = KeychainError(code: errSecItemNotFound)
    public static let interactionNotAllowed = KeychainError(code: errSecInteractionNotAllowed)

    public static func == (lhs: KeychainError, rhs: KeychainError) -> Bool {
        lhs.code == rhs.code
    }
}
