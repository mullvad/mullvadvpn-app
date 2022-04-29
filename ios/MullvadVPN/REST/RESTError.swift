//
//  RESTError.swift
//  RESTError
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {

    /// An error type returned by REST API classes.
    enum Error: ChainedError {
        /// A failure to encode the payload
        case encodePayload(Swift.Error)

        /// A failure during networking
        case network(URLError)

        /// A failure reported by server
        case server(REST.ServerErrorResponse)

        /// A failure to decode the error response from server
        case decodeErrorResponse(Swift.Error)

        /// A failure to decode the success response from server
        case decodeSuccessResponse(Swift.Error)

        var errorDescription: String? {
            switch self {
            case .encodePayload:
                return "Failure to encode the payload."
            case .network:
                return "Network error."
            case .server:
                return "Server error."
            case .decodeErrorResponse:
                return "Failure to decode error response from server."
            case .decodeSuccessResponse:
                return "Failure to decode success response from server."
            }
        }
    }

    /// A struct that represents a server response in case of error (any HTTP status code except 2xx).
    struct ServerErrorResponse: LocalizedError, Decodable, Equatable {
        /// A list of known server error codes
        enum Code: String, Equatable {
            case invalidAccount = "INVALID_ACCOUNT"
            case keyLimitReached = "KEY_LIMIT_REACHED"
            case pubKeyNotFound = "PUBKEY_NOT_FOUND"
            case invalidAccessToken = "INVALID_ACCESS_TOKEN"

            static func ~= (pattern: Self, value: REST.ServerErrorResponse) -> Bool {
                return pattern.rawValue == value.code
            }
        }

        static var invalidAccount: Code {
            return .invalidAccount
        }
        static var keyLimitReached: Code {
            return .keyLimitReached
        }
        static var pubKeyNotFound: Code {
            return .pubKeyNotFound
        }
        static var invalidAccessToken: Code {
            return .invalidAccessToken
        }

        let code: String
        let error: String?

        var errorDescription: String? {
            switch code {
            case Code.keyLimitReached.rawValue:
                return NSLocalizedString(
                    "KEY_LIMIT_REACHED_ERROR_DESCRIPTION",
                    tableName: "REST",
                    value: "Too many WireGuard keys in use.",
                    comment: ""
                )
            case Code.invalidAccount.rawValue:
                return NSLocalizedString(
                    "INVALID_ACCOUNT_ERROR_DESCRIPTION",
                    tableName: "REST",
                    value: "Invalid account.",
                    comment: ""
                )

            case Code.invalidAccessToken.rawValue:
                return NSLocalizedString(
                    "INVALID_ACCESS_TOKEN_ERROR_DESCRIPTION",
                    tableName: "REST",
                    value: "Invalid access token.",
                    comment: "")
            default:
                let localizedString = NSLocalizedString(
                    "UNKNOWN_ERROR_DESCRIPTION",
                    tableName: "REST",
                    value: "Unknown error: %@",
                    comment: "Use %@ placeholder to place the error code into the localized string."
                )
                return String(format: localizedString, code)
            }
        }

        var recoverySuggestion: String? {
            switch code {
            case Code.keyLimitReached.rawValue:
                return NSLocalizedString(
                    "KEY_LIMIT_REACHED_ERROR_RECOVERY_SUGGESTION",
                    tableName: "REST",
                    value: "Please visit the website to revoke a key before login is possible.",
                    comment: ""
                )
            default:
                return nil
            }
        }

        static func == (lhs: Self, rhs: Self) -> Bool {
            return lhs.code == rhs.code
        }
    }

}
