//
//  LegacyStoreSubscription.swift
//  MullvadVPN
//
//  Created by pronebird on 03/09/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import StoreKit

enum LegacyStoreSubscription: String, CaseIterable {
    case thirtyDays = "net.mullvad.MullvadVPN.subscription.30days"
    case ninetyDays = "net.mullvad.MullvadVPN.subscription.90days"

    var localizedTitle: String {
        switch self {
        case .thirtyDays:
            return NSLocalizedString("Add 30 days time (%@)", comment: "")
        case .ninetyDays:
            return NSLocalizedString("Add 90 days time (%@)", comment: "")
        }
    }
}

extension SKProduct {
    var customLocalizedTitle: String? {
        guard let localizedTitle = LegacyStoreSubscription(rawValue: productIdentifier)?.localizedTitle,
            let localizedPrice
        else {
            return nil
        }
        return String(format: localizedTitle, localizedPrice)
    }
}

extension Set<LegacyStoreSubscription> {
    var productIdentifiersSet: Set<String> {
        Set<String>(map { $0.rawValue })
    }
}
