//
//  AccessMethodValidationError.swift
//  MullvadVPN
//
//  Created by pronebird on 17/11/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Access method validation error that holds an array of individual per-field validation errors.
struct AccessMethodValidationError: LocalizedError, Equatable {
    /// The list of per-field errors.
    let fieldErrors: [AccessMethodFieldValidationError]

    var errorDescription: String? {
        if fieldErrors.count > 1 {
            NSLocalizedString(
                "VALIDATION_ERRORS_MULTIPLE",
                tableName: "APIAccess",
                value: "Multiple validation errors occurred.",
                comment: ""
            )
        } else {
            fieldErrors.first?.localizedDescription
        }
    }
}

/// Access method field validation error.
struct AccessMethodFieldValidationError: LocalizedError, Equatable {
    /// Validated field.
    enum Field: String, CustomStringConvertible, Equatable {
        case name, server, port, username, password

        var description: String {
            rawValue
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
    }

    /// Kind of validation error.
    let kind: Kind

    /// Error field.
    let field: Field

    /// Validation field context.
    let context: Context

    var errorDescription: String? {
        switch kind {
        case .emptyValue:
            NSLocalizedString(
                "VALIDATION_ERRORS_EMPTY_FIELD",
                tableName: "APIAccess",
                value: "\(field) cannot be empty.",
                comment: ""
            )
        case .invalidIPAddress:
            NSLocalizedString(
                "VALIDATION_ERRORS_INVALD ADDRESS",
                tableName: "APIAccess",
                value: "Please enter a valid IPv4 or IPv6 address.",
                comment: ""
            )
        case .invalidPort:
            NSLocalizedString(
                "VALIDATION_ERRORS_INVALID_PORT",
                tableName: "APIAccess",
                value: "Please enter a valid port.",
                comment: ""
            )
        case .nameTooLong:
            String(
                format: NSLocalizedString(
                    "VALIDATION_ERRORS_NAME_TOO_LONG",
                    tableName: "APIAccess",
                    value: "Name should be no longer than %i characters.",
                    comment: ""
                ),
                NameInputFormatter.maxLength
            )
        }
    }
}

extension Collection<AccessMethodFieldValidationError> {
    var settingsFieldValidationErrors: [SettingsFieldValidationError] {
        map { SettingsFieldValidationError(errorDescription: $0.errorDescription) }
    }
}
