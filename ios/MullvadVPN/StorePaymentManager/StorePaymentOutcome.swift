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

enum StorePaymentOutcome {
    case noTimeAdded
    case timeAdded(_ timeAdded: TimeInterval)

    var timeAdded: TimeInterval {
        switch self {
        case .noTimeAdded:
            return 0
        case let .timeAdded(timeAdded):
            return timeAdded
        }
    }

    /// Returns a formatted string for the `timeAdded` interval, i.e "30 days"
    var formattedTimeAdded: String? {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day]
        formatter.unitsStyle = .full

        return formatter.string(from: timeAdded)
    }

    func alertMessage(for context: Context) -> String {
        switch context {
        case .purchase, .restorationBeforePurchase:
            return String(
                format: NSLocalizedString("%@ have been added to your account.", comment: ""),
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
                return NSLocalizedString(
                    "Your previous purchases have been added to your account.",
                    comment: ""
                )
            }
        }
    }
}

extension StorePaymentOutcome {
    enum Context {
        case purchase
        case restoration
        case restorationBeforePurchase

        var alertTitle: String {
            switch self {
            case .purchase:
                return NSLocalizedString("Thanks for your purchase", comment: "")
            case .restoration:
                return NSLocalizedString("Restore purchases", comment: "")
            case .restorationBeforePurchase:
                return NSLocalizedString("Previous purchases were found", comment: "")
            }
        }

        var errorTitle: String {
            switch self {
            case .purchase:
                return NSLocalizedString("Cannot complete the purchase", comment: "")
            case .restoration:
                return NSLocalizedString("Cannot restore purchases", comment: "")
            case .restorationBeforePurchase:
                return NSLocalizedString("Previous purchases were found", comment: "")
            }
        }
    }
}
