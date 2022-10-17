//
//  AppStorePaymentManagerError.swift
//  AppStorePaymentManagerError
//
//  Created by pronebird on 08/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

public extension AppStorePaymentManager {
    /// An error type emitted by `AppStorePaymentManager`.
    enum Error: LocalizedError, WrappingError {
        /// Failure to find the account token associated with the transaction.
        case noAccountSet

        /// Failure to validate the account token.
        case validateAccount(REST.Error)

        /// Failure to handle payment transaction. Contains error returned by StoreKit.
        case storePayment(Swift.Error)

        /// Failure to read the AppStore receipt.
        case readReceipt(Swift.Error)

        /// Failure to send the AppStore receipt to backend.
        case sendReceipt(REST.Error)

        public var errorDescription: String? {
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

        public var underlyingError: Swift.Error? {
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
}
