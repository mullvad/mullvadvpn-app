//
//  AppStorePaymentManagerError.swift
//  AppStorePaymentManagerError
//
//  Created by pronebird on 08/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension AppStorePaymentManager {
    /// An error type emitted by `AppStorePaymentManager`.
    enum Error: ChainedError {
        /// Failure to find the account token associated with the transaction.
        case noAccountSet

        /// Failure to handle payment transaction. Contains error returned by StoreKit.
        case storePayment(Swift.Error)

        /// Failure to read the AppStore receipt.
        case readReceipt(AppStoreReceipt.Error)

        /// Failure to send the AppStore receipt to backend.
        case sendReceipt(REST.Error)

        var errorDescription: String? {
            switch self {
            case .noAccountSet:
                return "Account is not set"
            case .storePayment:
                return "Store payment error"
            case .readReceipt:
                return "Read recept error"
            case .sendReceipt:
                return "Send receipt error"
            }
        }
    }
}
