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
