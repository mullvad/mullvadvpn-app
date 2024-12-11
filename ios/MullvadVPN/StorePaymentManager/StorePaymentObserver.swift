//
//  StorePaymentObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 03/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol StorePaymentObserver: AnyObject, Sendable {
    func storePaymentManager(
        _ manager: StorePaymentManager,
        didReceiveEvent event: StorePaymentEvent
    )
}
