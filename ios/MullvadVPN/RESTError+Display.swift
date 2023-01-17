//
//  RESTError+Display.swift
//  MullvadVPN
//
//  Created by pronebird on 04/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

extension REST.Error: DisplayError {
    public var displayErrorDescription: String? {
        switch self {
        case let .network(urlError):
            return String(
                format: NSLocalizedString(
                    "NETWORK_ERROR",
                    tableName: "REST",
                    value: "Network error: %@",
                    comment: ""
                ),
                urlError.localizedDescription
            )

        case let .unhandledResponse(statusCode, serverResponse):
            guard let serverResponse = serverResponse else {
                return String(format: NSLocalizedString(
                    "INVALID_ACCOUNT_ERROR",
                    tableName: "REST",
                    value: "Unexpected server response: %@",
                    comment: ""
                ), statusCode)
            }

            if serverResponse.code == .invalidAccount {
                return NSLocalizedString(
                    "INVALID_ACCOUNT_ERROR",
                    tableName: "REST",
                    value: "Invalid account",
                    comment: ""
                )
            } else {
                return String(
                    format: NSLocalizedString(
                        "SERVER_ERROR",
                        tableName: "REST",
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
                tableName: "REST",
                value: "Internal error.",
                comment: ""
            )
        }
    }
}
