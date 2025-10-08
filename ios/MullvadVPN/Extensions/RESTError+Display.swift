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
                format: NSLocalizedString("Network error: %@", comment: ""),
                urlError.localizedDescription
            )

        case let .unhandledResponse(statusCode, serverResponse):
            guard let serverResponse else {
                return String(format: NSLocalizedString("Unexpected server response: %d", comment: ""), statusCode)
            }

            switch serverResponse.code {
            case .invalidAccount:
                return NSLocalizedString("Invalid account number", comment: "")

            case .maxDevicesReached:
                return NSLocalizedString("Too many devices", comment: "")

            case .serviceUnavailable:
                return NSLocalizedString("We are having some issues, please try again later", comment: "")

            case .tooManyRequests:
                return NSLocalizedString("We are having some issues, please try again later", comment: "")

            default:
                return String(
                    format: NSLocalizedString("Unexpected server response: %1$@ (HTTP status: %2$d)", comment: ""),
                    serverResponse.code.rawValue,
                    statusCode
                )
            }

        default:
            return NSLocalizedString("Internal error.", comment: "")
        }
    }
}
