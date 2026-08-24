// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
