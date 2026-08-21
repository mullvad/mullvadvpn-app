// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadREST

extension REST.CreateApplePaymentResponse {
    enum Context {
        case purchase
        case restoration
    }

    func alertTitle(context: Context) -> String {
        switch context {
        case .purchase:
            return NSLocalizedString("Thanks for your purchase", comment: "")
        case .restoration:
            return NSLocalizedString("Restore purchases", comment: "")
        }
    }

    func alertMessage(context: Context) -> String {
        switch context {
        case .purchase:
            return String(
                format: NSLocalizedString("%@ was added to your account.", comment: ""),
                formattedTimeAdded ?? ""
            )
        case .restoration:
            switch self {
            case .noTimeAdded:
                return NSLocalizedString(
                    "Your previous purchases have already been added to this account.",
                    comment: ""
                )
            case .timeAdded:
                return String(
                    format: NSLocalizedString("%@ was added to your account.", comment: ""),
                    formattedTimeAdded ?? ""
                )
            }
        }
    }
}

extension REST.CreateApplePaymentResponse.Context {
    var errorTitle: String {
        switch self {
        case .purchase:
            return NSLocalizedString("Cannot complete the purchase", comment: "")
        case .restoration:
            return NSLocalizedString("Cannot restore purchases", comment: "")
        }
    }
}
