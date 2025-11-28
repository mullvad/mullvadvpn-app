//
//  StorePaymentEvent.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
@preconcurrency import StoreKit

// MARK: StoreKit 2 flow

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
        case .unknown:
            NSLocalizedString("Unexpected error occured.", comment: "")
        case .receiptUpload:
            NSLocalizedString(
                "Failed to upload one or more receipts to Mullvad servers. Try again later or contact support for help.",
                comment: ""
            )
        }
    }
}

// MARK: Legacy StoreKit flow

/// The payment event received by observers implementing ``StorePaymentObserver``.
enum LegacyStorePaymentEvent: @unchecked Sendable {
    /// The payment is successfully completed.
    case finished(LegacyStorePaymentCompletion)

    /// Failure to complete the payment.
    case failure(LegacyStorePaymentFailure)

    /// An instance of `SKPayment` held in the associated value.
    var payment: SKPayment {
        switch self {
        case let .finished(completion):
            return completion.transaction.payment
        case let .failure(failure):
            return failure.payment
        }
    }
}

/// Successful payment metadata.
struct LegacyStorePaymentCompletion {
    /// Transaction object.
    let transaction: SKPaymentTransaction

    /// The account number credited.
    let accountNumber: String

    /// The server response received after uploading the App Store receipt.
    let serverResponse: REST.CreateApplePaymentResponse
}

/// Failed payment metadata.
struct LegacyStorePaymentFailure: @unchecked Sendable {
    /// Transaction object, if available.
    /// May not be available due to account validation failure.
    let transaction: SKPaymentTransaction?

    /// The payment object associated with payment request.
    let payment: SKPayment

    /// The account number to credit.
    /// May not be available if the payment manager couldn't establish the association between the payment and account number.
    /// Typically in such case, the error would be set to ``StorePaymentManagerError/noAccountSet``.
    let accountNumber: String?

    /// The payment manager error.
    let error: LegacyStorePaymentManagerError
}
