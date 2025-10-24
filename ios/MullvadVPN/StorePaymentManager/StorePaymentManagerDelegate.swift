//
//  StorePaymentManagerDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 03/09/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import StoreKit

protocol StorePaymentManagerDelegate: AnyObject, Sendable {
    /// Return the account number associated with the payment.
    /// Usually called for unfinished transactions coming back after the app was restarted.
    func fetchAccountToken(for payment: SKPayment) -> String?

    /// Called when the listener needs the current account number.
    func fetchAccountNumber() -> String?

    /// Called when the listener needs the current account number.
    func fetchAccountExpiry() -> Date?

    /// Called when account data has been successfully updated.
    func updateAccountData(for account: Account)
}
