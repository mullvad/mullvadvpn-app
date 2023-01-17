//
//  StorePaymentManagerError.swift
//  MullvadVPN
//
//  Created by pronebird on 08/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

/// An error type emitted by `StorePaymentManager`.
enum StorePaymentManagerError: LocalizedError, WrappingError {
    /// Failure to find the account token associated with the transaction.
    case noAccountSet

    /// Failure to validate the account token.
    case validateAccount(REST.Error)

    /// Failure to handle payment transaction. Contains error returned by StoreKit.
    case storePayment(Error)

    /// Failure to read the AppStore receipt.
    case readReceipt(Error)

    /// Failure to send the AppStore receipt to backend.
    case sendReceipt(REST.Error)

    var errorDescription: String? {
        switch self {
        case .noAccountSet:
            return "Account is not set."
        case .validateAccount:
            return "Account validation error."
        case .storePayment:
            return "Store payment error."
        case .readReceipt:
            return "Read recept error."
        case .sendReceipt:
            return "Send receipt error."
        }
    }

    var underlyingError: Error? {
        switch self {
        case .noAccountSet:
            return nil
        case let .sendReceipt(error):
            return error
        case let .validateAccount(error):
            return error
        case let .readReceipt(error):
            return error
        case let .storePayment(error):
            return error
        }
    }
}
