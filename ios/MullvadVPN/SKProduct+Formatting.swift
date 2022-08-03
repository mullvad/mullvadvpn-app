//
//  SKProduct+Formatting.swift
//  MullvadVPN
//
//  Created by pronebird on 19/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

extension SKProduct {
    var localizedPrice: String? {
        let formatter = NumberFormatter()
        formatter.locale = priceLocale
        formatter.numberStyle = .currency

        return formatter.string(from: price)
    }
}
