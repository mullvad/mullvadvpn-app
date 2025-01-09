//
//  AccountInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations
import StoreKit

final class AccountInteractor: Sendable {
    private let storePaymentManager: StorePaymentManager
    let tunnelManager: TunnelManager
    let accountsProxy: RESTAccountHandling

    nonisolated(unsafe) var didReceivePaymentEvent: (@Sendable (StorePaymentEvent) -> Void)?
    nonisolated(unsafe) var didReceiveDeviceState: (@Sendable (DeviceState) -> Void)?

    nonisolated(unsafe) private var tunnelObserver: TunnelObserver?
    nonisolated(unsafe) private var paymentObserver: StorePaymentObserver?

    init(
        storePaymentManager: StorePaymentManager,
        tunnelManager: TunnelManager,
        accountsProxy: RESTAccountHandling
    ) {
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager
        self.accountsProxy = accountsProxy

        let tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] _, deviceState, _ in
                self?.didReceiveDeviceState?(deviceState)
            })

        let paymentObserver = StorePaymentBlockObserver { [weak self] _, event in
            self?.didReceivePaymentEvent?(event)
        }

        tunnelManager.addObserver(tunnelObserver)
        storePaymentManager.addPaymentObserver(paymentObserver)

        self.tunnelObserver = tunnelObserver
        self.paymentObserver = paymentObserver
    }

    var deviceState: DeviceState {
        tunnelManager.deviceState
    }

    func logout() async {
        await tunnelManager.unsetAccount()
    }

    func addPayment(_ payment: SKPayment, for accountNumber: String) {
        storePaymentManager.addPayment(payment, for: accountNumber)
    }

    func restorePurchases(
        for accountNumber: String,
        completionHandler: @escaping @Sendable (Result<REST.CreateApplePaymentResponse, Error>) -> Void
    ) -> Cancellable {
        storePaymentManager.restorePurchases(
            for: accountNumber,
            completionHandler: completionHandler
        )
    }

    func requestProducts(
        with productIdentifiers: Set<StoreSubscription>,
        completionHandler: @escaping @Sendable (Result<SKProductsResponse, Error>) -> Void
    ) -> Cancellable {
        storePaymentManager.requestProducts(
            with: productIdentifiers,
            completionHandler: completionHandler
        )
    }
}
