//
//  PaymentState.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

enum PaymentState: Equatable {
    case none
    case makingPayment(SKPayment)
    case makingStoreKit2Purchase
    case restoringPurchases

    var allowsViewInteraction: Bool {
        switch self {
        case .none:
            return true
        case .restoringPurchases, .makingPayment, .makingStoreKit2Purchase:
            return false
        }
    }
}
