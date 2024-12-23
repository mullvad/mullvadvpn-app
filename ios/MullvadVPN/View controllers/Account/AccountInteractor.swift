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

final class AccountInteractor {
    private let storePaymentManager: StorePaymentManager
    let tunnelManager: TunnelManager
    let accountsProxy: RESTAccountHandling
    let apiProxy: APIQuerying

    var didReceivePaymentEvent: ((StorePaymentEvent) -> Void)?
    var didReceiveDeviceState: ((DeviceState) -> Void)?

    private var tunnelObserver: TunnelObserver?
    private var paymentObserver: StorePaymentObserver?

    init(
        storePaymentManager: StorePaymentManager,
        tunnelManager: TunnelManager,
        accountsProxy: RESTAccountHandling,
        apiProxy: APIQuerying
    ) {
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager
        self.accountsProxy = accountsProxy
        self.apiProxy = apiProxy

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

    func sendStoreKitReceipt(_ transaction: VerificationResult<Transaction>, for accountNumber: String) async throws {
        try await apiProxy.createApplePayment(
            accountNumber: accountNumber,
            receiptString: transaction.jwsRepresentation.data(using: .utf8)!
        ).execute()
    }

    func restorePurchases(
        for accountNumber: String,
        completionHandler: @escaping (Result<REST.CreateApplePaymentResponse, Error>) -> Void
    ) -> Cancellable {
        storePaymentManager.restorePurchases(
            for: accountNumber,
            completionHandler: completionHandler
        )
    }

    func requestProducts(
        with productIdentifiers: Set<StoreSubscription>,
        completionHandler: @escaping (Result<SKProductsResponse, Error>) -> Void
    ) -> Cancellable {
        storePaymentManager.requestProducts(
            with: productIdentifiers,
            completionHandler: completionHandler
        )
    }
}
