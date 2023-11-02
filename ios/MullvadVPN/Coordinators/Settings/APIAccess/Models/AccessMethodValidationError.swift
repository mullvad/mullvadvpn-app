//
//  AccessMethodValidationError.swift
//  MullvadVPN
//
//  Created by pronebird on 17/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Access method validation error that holds an array of individual per-field validation errors.
struct AccessMethodValidationError: LocalizedError, Equatable {
    /// The list of per-field errors.
    let fieldErrors: [AccessMethodFieldValidationError]

    var errorDescription: String? {
        if fieldErrors.count > 1 {
            "Multiple validation errors occurred."
        } else {
            fieldErrors.first?.localizedDescription
        }
    }
}

/// Access method field validation error.
struct AccessMethodFieldValidationError: LocalizedError, Equatable {
    /// Validated field.
    enum Field: String, CustomStringConvertible, Equatable {
        case server, port, username

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
        case parseIPAddress

        /// Failure to parse port value.
        case parsePort

        /// Invalid port number, i.e zero.
        case invalidPort
    }

    /// Kind of validation error.
    let kind: Kind

    /// Error field.
    let field: Field

    /// Validation field context.
    let context: Context

    var errorDescription: String? {
        var s = "The \(context) \(field) "
        switch kind {
        case .emptyValue:
            s += "cannot be empty."
        case .parseIPAddress:
            s += "cannot be parsed as IP address."
        case .parsePort:
            s += "cannot be parsed as a port number."
        case .invalidPort:
            s += "contains invalid port number."
        }
        return s
    }
}
