// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
