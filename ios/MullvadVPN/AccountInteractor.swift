//
//  AccountInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import Operations
import StoreKit

final class AccountInteractor {
    private let storePaymentManager: StorePaymentManager
    private let tunnelManager: TunnelManager

    var didReceivePaymentEvent: ((StorePaymentEvent) -> Void)?
    var didReceiveDeviceState: ((DeviceState) -> Void)?

    private var tunnelObserver: TunnelObserver?
    private var paymentObserver: StorePaymentObserver?

    init(storePaymentManager: StorePaymentManager, tunnelManager: TunnelManager) {
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager

        let tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] manager, deviceState in
                self?.didReceiveDeviceState?(deviceState)
            })

        let paymentObserver = StorePaymentBlockObserver { [weak self] manager, event in
            self?.didReceivePaymentEvent?(event)
        }

        tunnelManager.addObserver(tunnelObserver)
        storePaymentManager.addPaymentObserver(paymentObserver)

        self.tunnelObserver = tunnelObserver
        self.paymentObserver = paymentObserver
    }

    var deviceState: DeviceState {
        return tunnelManager.deviceState
    }

    func logout(_ completion: @escaping () -> Void) {
        tunnelManager.unsetAccount(completionHandler: completion)
    }

    func addPayment(_ payment: SKPayment, for accountNumber: String) {
        storePaymentManager.addPayment(payment, for: accountNumber)
    }

    func restorePurchases(
        for accountNumber: String,
        completionHandler: @escaping (OperationCompletion<
            REST.CreateApplePaymentResponse,
            StorePaymentManagerError
        >) -> Void
    ) -> Cancellable {
        return storePaymentManager.restorePurchases(
            for: accountNumber,
            completionHandler: completionHandler
        )
    }

    func requestProducts(
        with productIdentifiers: Set<StoreSubscription>,
        completionHandler: @escaping (OperationCompletion<SKProductsResponse, Error>) -> Void
    ) -> Cancellable {
        return storePaymentManager.requestProducts(
            with: productIdentifiers,
            completionHandler: completionHandler
        )
    }
}
