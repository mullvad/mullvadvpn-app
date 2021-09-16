//
//  AnyAppStorePaymentObserver.swift
//  AnyAppStorePaymentObserver
//
//  Created by pronebird on 03/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

/// A type-erasing weak container for `AppStorePaymentObserver`
class AnyAppStorePaymentObserver: AppStorePaymentObserver, WeakObserverBox, Equatable {
    private(set) weak var inner: AppStorePaymentObserver?

    init<T: AppStorePaymentObserver>(_ inner: T) {
        self.inner = inner
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager,
                                transaction: SKPaymentTransaction,
                                accountToken: String?,
                                didFailWithError error: AppStorePaymentManager.Error)
    {
        self.inner?.appStorePaymentManager(
            manager,
            transaction: transaction,
            accountToken: accountToken,
            didFailWithError: error)
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager,
                                transaction: SKPaymentTransaction,
                                accountToken: String,
                                didFinishWithResponse response: REST.CreateApplePaymentResponse)
    {
        self.inner?.appStorePaymentManager(
            manager,
            transaction: transaction,
            accountToken: accountToken,
            didFinishWithResponse: response)
    }

    static func == (lhs: AnyAppStorePaymentObserver, rhs: AnyAppStorePaymentObserver) -> Bool {
        return lhs.inner === rhs.inner
    }
}
