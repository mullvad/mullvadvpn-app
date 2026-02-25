//
//  StorePaymentEvent.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import StoreKit

enum StorePaymentEvent {
    /// Successful payment
    case successfulPayment(StorePaymentOutcome)
    /// Use cancelled the purchase
    case userCancelled
    /// Payment was made but it is still being processed. This transaction can be processed and the receipt uploaded to the API later, when the transaction listener handles it.
    case pending
    /// Purchasing failed
    case failed(StorePaymentError)
}

enum StorePaymentError: Error {
    /// Purchase failed because the product being purchased is either unavailable or StoreKit services failed.
    case storeKitError(StoreKitError)
    /// Purchase failed because of a "purchase error".
    case purchaseError(Product.PurchaseError)
    /// User made a purchase, but we failed to verify the transaction. In this case, it is fine to not send the transaction to the API.
    case verification(VerificationResult<Transaction>.VerificationError)
    /// In this case, the user has initiated the payment but the app failed to fetch a payment token from the API.
    /// No money has been spent and the payment has failed.
    case getPaymentToken(Error)
    /// In this case, the user has already spent money but we failed to upload the receipt to the API.
    /// They should be fine as the API should , but we can still upload the receipt later
    case receiptUpload
    /// Purchase restoration was unsuccessful.
    case restorationError
    /// To handle errors we don't recognize.
    case unknown

    var description: String {
        switch self {
        case let .storeKitError(error):
            error.localizedDescription
        case let .purchaseError(error):
            error.localizedDescription
        case .verification:
            NSLocalizedString("Failed to verify transaction receipt", comment: "")
        case .getPaymentToken:
            NSLocalizedString("Failed to reach Mullvad servers to initiate purchase", comment: "")
        case .receiptUpload:
            NSLocalizedString(
                "Failed to upload one or more receipts to Mullvad servers. Try again later or contact support for help.",
                comment: ""
            )
        case .restorationError:
            NSLocalizedString(
                "Could not restore previous purchases. Try again later or contact support.",
                comment: ""
            )
        case .unknown:
            NSLocalizedString("Unexpected error occured.", comment: "")
        }
    }
}
