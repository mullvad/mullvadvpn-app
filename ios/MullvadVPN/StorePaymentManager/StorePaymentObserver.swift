//
//  StorePaymentObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 03/09/2021.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

protocol StorePaymentObserver: AnyObject, Sendable {
    @MainActor func storePaymentManager(didReceiveEvent event: StorePaymentEvent)
}
