//
//  AppStoreSubscription.swift
//  AppStoreSubscription
//
//  Created by pronebird on 03/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

enum AppStoreSubscription: String {
    /// Thirty days non-renewable subscription
    case thirtyDays = "net.mullvad.MullvadVPN.subscription.30days"

    var localizedTitle: String {
        switch self {
        case .thirtyDays:
            return NSLocalizedString(
                "APPSTORE_SUBSCRIPTION_TITLE_ADD_30_DAYS",
                tableName: "AppStoreSubscriptions",
                value: "Add 30 days time",
                comment: ""
            )
        }
    }
}

extension SKProduct {
    var customLocalizedTitle: String? {
        return AppStoreSubscription(rawValue: productIdentifier)?.localizedTitle
    }
}

extension Set where Element == AppStoreSubscription {
    var productIdentifiersSet: Set<String> {
        Set<String>(map { $0.rawValue })
    }
}
