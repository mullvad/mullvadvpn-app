//
//  RESTError+Display.swift
//  MullvadVPN
//
//  Created by pronebird on 04/06/2020.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

extension REST.Error: MullvadTypes.DisplayError {
    public var displayErrorDescription: String? {
        switch self {
        case let .network(urlError):
            return String(
                format: NSLocalizedString(
                    "NETWORK_ERROR",
                    value: "Network error: %@",
                    comment: ""
                ),
                urlError.localizedDescription
            )

        case let .unhandledResponse(statusCode, serverResponse):
            guard let serverResponse else {
                return String(format: NSLocalizedString(
                    "UNEXPECTED_RESPONSE",
                    value: "Unexpected server response: %d",
                    comment: ""
                ), statusCode)
            }

            switch serverResponse.code {
            case .invalidAccount:
                return NSLocalizedString(
                    "INVALID_ACCOUNT_ERROR",
                    value: "Invalid account",
                    comment: ""
                )

            case .maxDevicesReached:
                return NSLocalizedString(
                    "MAX_DEVICES_REACHED_ERROR",
                    value: "Too many devices registered with account",
                    comment: ""
                )

            case .serviceUnavailable:
                return NSLocalizedString(
                    "SERVICE_UNAVAILABLE",
                    value: "We are having some issues, please try again later",
                    comment: ""
                )

            case .tooManyRequests:
                return NSLocalizedString(
                    "TOO_MANY_REQUESTS",
                    value: "We are having some issues, please try again later",
                    comment: ""
                )

            default:
                return String(
                    format: NSLocalizedString(
                        "SERVER_ERROR",
                        value: "Unexpected server response: %1$@ (HTTP status: %2$d)",
                        comment: ""
                    ),
                    serverResponse.code.rawValue,
                    statusCode
                )
            }

        default:
            return NSLocalizedString(
                "INTERNAL_ERROR",
                value: "Internal error.",
                comment: ""
            )
        }
    }
}
