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
private let accountUpdateTimerInterval: TimeInterval = 60

final class OutOfTimeInteractor {
    private let storePaymentManager: StorePaymentManager
    private let tunnelManager: TunnelManager

    private var tunnelObserver: TunnelObserver?
    private var paymentObserver: StorePaymentObserver?

    private let logger = Logger(label: "OutOfTimeInteractor")
    private var accountUpdateTimer: DispatchSourceTimer?
    private var isPolling = false

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
        return tunnelManager.tunnelStatus
    }

    var deviceState: DeviceState {
        return tunnelManager.deviceState
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
        return storePaymentManager.restorePurchases(
            for: accountNumber,
            completionHandler: completionHandler
        )
    }

    func requestProducts(
        with productIdentifiers: Set<StoreSubscription>,
        completionHandler: @escaping (Result<SKProductsResponse, Error>) -> Void
    ) -> Cancellable {
        return storePaymentManager.requestProducts(
            with: productIdentifiers,
            completionHandler: completionHandler
        )
    }

    func startAccountUpdateTimer() {
        guard !isPolling else { return }
        isPolling = true

        logger.debug(
            "Start polling account updates every \(accountUpdateTimerInterval) second(s)."
        )

        let timer = DispatchSource.makeTimerSource(queue: .main)
        timer.setEventHandler { [weak self] in
            self?.tunnelManager.updateAccountData()
        }
        timer.schedule(wallDeadline: .now() + accountUpdateTimerInterval, repeating: accountUpdateTimerInterval)
        timer.activate()

        accountUpdateTimer?.cancel()
        accountUpdateTimer = timer
    }
}
