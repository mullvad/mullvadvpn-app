//
//  StorePaymentObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 03/09/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

protocol StorePaymentObserver: AnyObject, Sendable {
    @MainActor func storePaymentManager(didReceiveEvent event: StorePaymentEvent)
    @MainActor func storePaymentManager(didReceiveEvent event: LegacyStorePaymentEvent)
}
