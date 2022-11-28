//
//  ProductState.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

enum ProductState {
    case none
    case fetching(StoreSubscription)
    case received(SKProduct)
    case failed
    case cannotMakePurchases

    var isFetching: Bool {
        if case .fetching = self {
            return true
        }
        return false
    }

    var isReceived: Bool {
        if case .received = self {
            return true
        }
        return false
    }

    var purchaseButtonTitle: String? {
        switch self {
        case .none:
            return nil

        case let .fetching(subscription):
            return subscription.localizedTitle

        case let .received(product):
            let localizedTitle = product.customLocalizedTitle ?? ""
            let localizedPrice = product.localizedPrice ?? ""

            let format = NSLocalizedString(
                "PURCHASE_BUTTON_TITLE_FORMAT",
                tableName: "Account",
                value: "%1$@ (%2$@)",
                comment: ""
            )
            return String(format: format, localizedTitle, localizedPrice)

        case .failed:
            return NSLocalizedString(
                "PURCHASE_BUTTON_CANNOT_CONNECT_TO_APPSTORE_LABEL",
                tableName: "Account",
                value: "Cannot connect to AppStore",
                comment: ""
            )

        case .cannotMakePurchases:
            return NSLocalizedString(
                "PURCHASE_BUTTON_PAYMENTS_RESTRICTED_LABEL",
                tableName: "Account",
                value: "Payments restricted",
                comment: ""
            )
        }
    }
}
