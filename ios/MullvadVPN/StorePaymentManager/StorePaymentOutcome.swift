//
//  StorePaymentOutcome.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-10-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST

enum StorePaymentOutcome {
    case noTimeAdded
    case timeAdded(_ timeAdded: TimeInterval)

    var timeAdded: TimeInterval {
        switch self {
        case .noTimeAdded:
            return 0
        case let .timeAdded(timeAdded):
            return TimeInterval(timeAdded)
        }
    }

    /// Returns a formatted string for the `timeAdded` interval, i.e "30 days"
    var formattedTimeAdded: String? {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day, .hour]
        formatter.unitsStyle = .full

        return formatter.string(from: timeAdded)
    }

    func alertMessage(for context: Context) -> String {
        switch context {
        case .purchase:
            return String(
                format: NSLocalizedString("%@ have been added to your account", comment: ""),
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
                    format: NSLocalizedString("%@ have been added to your account", comment: ""),
                    formattedTimeAdded ?? ""
                )
            }
        }
    }
}

extension StorePaymentOutcome {
    enum Context {
        case purchase
        case restoration

        var alertTitle: String {
            switch self {
            case .purchase:
                return NSLocalizedString("Thanks for your purchase", comment: "")
            case .restoration:
                return NSLocalizedString("Restore purchases", comment: "")
            }
        }

        var errorTitle: String {
            switch self {
            case .purchase:
                return NSLocalizedString("Cannot complete the purchase", comment: "")
            case .restoration:
                return NSLocalizedString("Cannot restore purchases", comment: "")
            }
        }
    }
}
