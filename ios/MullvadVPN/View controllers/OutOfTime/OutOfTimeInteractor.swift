//
//  OutOfTimeInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import StoreKit

/// Interval used for periodic polling account updates.
private let accountUpdateTimerInterval: TimeInterval = .minutes(1)

final class OutOfTimeInteractor {
    private let storePaymentManager: StorePaymentManager
    private let tunnelManager: TunnelManager

    private var tunnelObserver: TunnelObserver?
    private var paymentObserver: StorePaymentObserver?

    private let logger = Logger(label: "OutOfTimeInteractor")
    private var accountUpdateTimer: DispatchSourceTimer?

    var didReceivePaymentEvent: ((StorePaymentEvent) -> Void)?
    var didReceiveTunnelStatus: ((TunnelStatus) -> Void)?

    init(storePaymentManager: StorePaymentManager, tunnelManager: TunnelManager) {
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager

        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { [weak self] manager, tunnelStatus in
                self?.didReceiveTunnelStatus?(tunnelStatus)
            }
        )

        let paymentObserver = StorePaymentBlockObserver { [weak self] manager, event in
            self?.didReceivePaymentEvent?(event)
        }

        tunnelManager.addObserver(tunnelObserver)
        storePaymentManager.addPaymentObserver(paymentObserver)

        self.tunnelObserver = tunnelObserver
        self.paymentObserver = paymentObserver
    }

    var tunnelStatus: TunnelStatus {
        tunnelManager.tunnelStatus
    }

    var deviceState: DeviceState {
        tunnelManager.deviceState
    }

    func stopTunnel() {
        tunnelManager.stopTunnel()
    }

    func addPayment(_ payment: SKPayment, for accountNumber: String) {
        storePaymentManager.addPayment(payment, for: accountNumber)
    }

    func restorePurchases(
        for accountNumber: String,
        completionHandler: @escaping (Result<
            REST.CreateApplePaymentResponse,
            Error
        >) -> Void
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

    func startAccountUpdateTimer() {
        logger.debug(
            "Start polling account updates every \(accountUpdateTimerInterval) second(s)."
        )

        let timer = DispatchSource.makeTimerSource(queue: .main)
        timer.setEventHandler { [weak self] in
            self?.tunnelManager.updateAccountData()
        }

        accountUpdateTimer?.cancel()
        accountUpdateTimer = timer

        timer.schedule(wallDeadline: .now() + accountUpdateTimerInterval, repeating: accountUpdateTimerInterval)
        timer.activate()
    }
}
