//
//  SKError+Localized.swift
//  MullvadVPN
//
//  Created by pronebird on 17/01/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import StoreKit

extension SKError: Foundation.LocalizedError {
    public var errorDescription: String? {
        switch code {
        case .unknown:
            return NSLocalizedString(
                "UNKNOWN_ERROR",
                tableName: "Payment",
                value: "Unknown error.",
                comment: ""
            )
        case .clientInvalid:
            return NSLocalizedString(
                "CLIENT_INVALID",
                tableName: "Payment",
                value: "Client is not allowed to issue the request.",
                comment: ""
            )
        case .paymentCancelled:
            return NSLocalizedString(
                "PAYMENT_CANCELLED",
                tableName: "Payment",
                value: "The payment request was cancelled.",
                comment: ""
            )
        case .paymentInvalid:
            return NSLocalizedString(
                "PAYMENT_INVALID",
                tableName: "Payment",
                value: "Invalid purchase identifier.",
                comment: ""
            )
        case .paymentNotAllowed:
            return NSLocalizedString(
                "PAYMENT_NOT_ALLOWED",
                tableName: "Payment",
                value: "This device is not allowed to make the payment.",
                comment: ""
            )
        default:
            return localizedDescription
        }
    }
}
