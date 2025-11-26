//
//  StoreSubscription.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-11-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import StoreKit

enum StoreSubscription: String, CaseIterable {
    case thirtyDays = "net.mullvad.MullvadVPN.subscription.storekit2.30days"
    case ninetyDays = "net.mullvad.MullvadVPN.subscription.storekit2.90days"

    func localizedTitle(displayPrice: String) -> String {
        switch self {
        case .thirtyDays:
            String(format: NSLocalizedString("Add 30 days time (%@)", comment: ""), displayPrice)
        case .ninetyDays:
            String(format: NSLocalizedString("Add 90 days time (%@)", comment: ""), displayPrice)
        }
    }
}

extension Product {
    var customLocalizedTitle: String? {
        StoreSubscription(rawValue: id)?.localizedTitle(displayPrice: displayPrice)
    }
}
