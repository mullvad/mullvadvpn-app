//
//  StorePaymentEvent.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import StoreKit

/// The payment event received by observers implementing ``StorePaymentObserver``.
enum StorePaymentEvent {
    /// The payment is successfully completed.
    case finished(StorePaymentCompletion)

    /// Failure to complete the payment.
    case failure(StorePaymentFailure)

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
struct StorePaymentCompletion {
    /// Transaction object.
    let transaction: SKPaymentTransaction

    /// The account number credited.
    let accountNumber: String

    /// The server response received after uploading the AppStore receipt.
    let serverResponse: REST.CreateApplePaymentResponse
}

/// Failed payment metadata.
struct StorePaymentFailure {
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
    let error: StorePaymentManagerError
}
