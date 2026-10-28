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
import MullvadTypes

/// Access method validation error that holds an array of individual per-field validation errors.
struct AccessMethodValidationError: LocalizedError, Equatable {
    /// The list of per-field errors.
    let fieldErrors: [AccessMethodFieldValidationError]

    var errorDescription: String? {
        fieldErrors
            .sorted { $0.field < $1.field }
            .compactMap(\.errorDescription)
            .joinedParagraphs(lineBreaks: 1)
    }
}

/// Access method field validation error.
struct AccessMethodFieldValidationError: Error, Equatable {
    /// Validated field.
    enum Field: String, CustomStringConvertible, Equatable, Comparable, CaseIterable {
        case name, server, port, username, password, cipher

        var description: String {
            rawValue
        }
        static func < (lhs: Self, rhs: Self) -> Bool {
            allCases.firstIndex(of: lhs)! < allCases.firstIndex(of: rhs)!
        }
    }

    /// Validated field context.
    enum Context: String, CustomStringConvertible, Equatable {
        case socks, shadowsocks

        var description: String {
            rawValue
        }
    }

    /// Validation error kind.
    enum Kind: Equatable {
        /// The evaluated field is empty.
        case emptyValue

        /// Failure to parse IP address.
        case invalidIPAddress

        /// Invalid port number, i.e zero.
        case invalidPort

        /// The name input is too long.
        case nameTooLong

        /// Invalid cipher, i.e no longer supported.
        case invalidCipher
    }

    /// Kind of validation error.
    let kind: Kind

    /// Error field.
    let field: Field

    /// Validation field context.
    let context: Context

    var errorDescription: String {
        switch kind {
        case .emptyValue:
            String(format: NSLocalizedString("%@ cannot be empty.", comment: ""), field.rawValue.localizedCapitalized)
        case .invalidIPAddress:
            NSLocalizedString("Enter a valid IPv4 or IPv6 address.", comment: "")
        case .invalidPort:
            NSLocalizedString("Enter a valid remote server port.", comment: "")
        case .nameTooLong:
            String(
                format: NSLocalizedString("Name should be no longer than %i characters", comment: ""),
                NameInputFormatter.maxLength
            )
        case .invalidCipher:
            NSLocalizedString("Selected cipher no longer supported", comment: "")
        }
    }
}

extension Collection<AccessMethodFieldValidationError> {
    var settingsFieldValidationErrors: [SettingsFieldValidationError] {
        map { SettingsFieldValidationError(errorDescription: $0.errorDescription) }
    }
}
