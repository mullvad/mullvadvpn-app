//
//  AppStorePaymentObserver.swift
//  AppStorePaymentObserver
//
//  Created by pronebird on 03/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import StoreKit

enum CreateApplePaymentResponse {
    case noTimeAdded(_ expiry: Date)
    case timeAdded(_ timeAdded: Int, _ newExpiry: Date)

    public var newExpiry: Date {
        switch self {
        case let .noTimeAdded(expiry), let .timeAdded(_, expiry):
            return expiry
        }
    }

    public var timeAdded: TimeInterval {
        switch self {
        case .noTimeAdded:
            return 0
        case let .timeAdded(timeAdded, _):
            return TimeInterval(timeAdded)
        }
    }

    /// Returns a formatted string for the `timeAdded` interval, i.e "30 days"
    public var formattedTimeAdded: String? {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day, .hour]
        formatter.unitsStyle = .full

        return formatter.string(from: self.timeAdded)
    }
}

public protocol AppStorePaymentObserver: AnyObject {
    func appStorePaymentManager(
        _ manager: AppStorePaymentManager,
        transaction: SKPaymentTransaction?,
        payment: SKPayment,
        accountToken: String?,
        didFailWithError error: AppStorePaymentManager.Error
    )

    func appStorePaymentManager(
        _ manager: AppStorePaymentManager,
        transaction: SKPaymentTransaction,
        accountToken: String,
        didFinishWithResponse response: REST.CreateApplePaymentResponse
    )
}
