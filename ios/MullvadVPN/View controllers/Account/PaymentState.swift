//
//  PaymentState.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

enum PaymentState: Equatable {
    case none
    case makingPurchase
    case makingRefund
    case restoringPurchases

    var allowsViewInteraction: Bool {
        switch self {
        case .none:
            return true
        case .restoringPurchases, .makingPurchase, .makingRefund:
            return false
        }
    }
}
