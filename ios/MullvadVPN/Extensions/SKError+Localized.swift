//
//  SKError+Localized.swift
//  MullvadVPN
//
//  Created by pronebird on 17/01/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import StoreKit

extension SKError: Foundation.LocalizedError {
    public var errorDescription: String? {
        switch code {
        case .unknown:
            return NSLocalizedString("Unknown error.", comment: "")
        case .clientInvalid:
            return NSLocalizedString("Client is not allowed to issue the request.", comment: "")
        case .paymentCancelled:
            return NSLocalizedString("The payment request was cancelled.", comment: "")
        case .paymentInvalid:
            return NSLocalizedString("Invalid purchase identifier.", comment: "")
        case .paymentNotAllowed:
            return NSLocalizedString("This device is not allowed to make the payment.", comment: "")
        default:
            return localizedDescription
        }
    }
}
