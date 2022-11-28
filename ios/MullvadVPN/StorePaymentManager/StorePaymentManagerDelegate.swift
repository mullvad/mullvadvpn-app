//
//  StorePaymentManagerDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 03/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

protocol StorePaymentManagerDelegate: AnyObject {
    /// Return the account token associated with the payment.
    /// Usually called for unfinished transactions coming back after the app was restarted.
    func storePaymentManager(
        _ manager: StorePaymentManager,
        didRequestAccountTokenFor payment: SKPayment
    ) -> String?
}
