//
//  StoreSubscription.swift
//  MullvadVPN
//
//  Created by pronebird on 03/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

enum StoreSubscription: String {
    /// Thirty days non-renewable subscription
    case thirtyDays = "net.mullvad.MullvadVPN.subscription.30days"

    var localizedTitle: String {
        switch self {
        case .thirtyDays:
            return NSLocalizedString(
                "STORE_SUBSCRIPTION_TITLE_ADD_30_DAYS",
                tableName: "StoreSubscriptions",
                value: "Add 30 days time",
                comment: ""
            )
        }
    }
}

extension SKProduct {
    var customLocalizedTitle: String? {
        return StoreSubscription(rawValue: productIdentifier)?.localizedTitle
    }
}

extension Set where Element == StoreSubscription {
    var productIdentifiersSet: Set<String> {
        return Set<String>(map { $0.rawValue })
    }
}
