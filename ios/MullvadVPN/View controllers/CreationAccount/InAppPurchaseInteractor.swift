//
//  InAppPurchaseInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-21.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

protocol InAppPurchaseViewControllerDelegate: AnyObject {
    func didBeginPayment()
    func didEndPayment()
}

class InAppPurchaseInteractor {
    let storePaymentManager: StorePaymentManager
    let tunnelManager: TunnelManager
    var didFinishPayment: ((InAppPurchaseInteractor, StorePaymentEvent) -> Void)?
    var didReceiveDeviceState: ((InAppPurchaseInteractor, DeviceState) -> Void)?
    weak var viewControllerDelegate: InAppPurchaseViewControllerDelegate?

    private var tunnelObserver: TunnelObserver?
    private var paymentObserver: StorePaymentObserver?

    init(storePaymentManager: StorePaymentManager, tunnelManager: TunnelManager) {
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager
        self.addObservers()
    }

    private func addObservers() {
        let tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] manager, deviceState, previousDeviceState in
                guard let self else { return }
                self.didReceiveDeviceState?(self, deviceState)
            })

        let paymentObserver = StorePaymentBlockObserver { [weak self] manager, event in
            guard let self else { return }
            viewControllerDelegate?.didBeginPayment()
            didFinishPayment?(self, event)
        }

        tunnelManager.addObserver(tunnelObserver)
        storePaymentManager.addPaymentObserver(paymentObserver)

        self.tunnelObserver = tunnelObserver
        self.paymentObserver = paymentObserver
    }

    func purchase(accountNumber: String, product: SKProduct) {
        self.viewControllerDelegate?.didBeginPayment()
        let payment = SKPayment(product: product)
        storePaymentManager.addPayment(payment, for: accountNumber)
    }
}
