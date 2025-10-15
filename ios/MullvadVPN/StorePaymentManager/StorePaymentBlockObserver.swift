//
//  StorePaymentBlockObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

final class StorePaymentBlockObserver: StorePaymentObserver {
    typealias BlockHandler = @Sendable (StorePaymentManager, StorePaymentEvent) -> Void

    private let blockHandler: BlockHandler

    init(_ blockHandler: @escaping BlockHandler) {
        self.blockHandler = blockHandler
    }

    func storePaymentManager(
        _ manager: StorePaymentManager,
        didReceiveEvent event: StorePaymentEvent
    ) {
        blockHandler(manager, event)
    }
    
    func storePaymentManager(
        _ manager: StorePaymentManager,
        didReceiveEvent event: StoreKitPaymentEvent,
    ) {
        // TODO
    }
}
