//
//  SKProduct+Sorting.swift
//  MullvadVPN
//
//  Created by Steffen Ernst on 2025-01-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import StoreKit

extension Array where Element == SKProduct {
    func sortedByPrice() -> [SKProduct] {
        sorted { ($0.price as Decimal) < ($1.price as Decimal) }
    }
}
