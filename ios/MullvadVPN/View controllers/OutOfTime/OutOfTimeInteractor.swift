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
import MullvadSettings
import MullvadTypes
import Operations
@preconcurrency import StoreKit

final class OutOfTimeInteractor: Sendable {
    private let storePaymentManager: StorePaymentManager
    private let tunnelManager: TunnelManager

    nonisolated(unsafe) private var tunnelObserver: TunnelObserver?
    nonisolated(unsafe) private var paymentObserver: StorePaymentObserver?

    nonisolated(unsafe) private let logger = Logger(label: "OutOfTimeInteractor")

    private let accountUpdateTimerInterval: Duration = .minutes(1)
    nonisolated(unsafe) private var accountUpdateTimer: DispatchSourceTimer?

    nonisolated(unsafe) var didReceivePaymentEvent: (@Sendable (StorePaymentEvent) -> Void)?
    nonisolated(unsafe) var didReceiveTunnelStatus: (@Sendable (TunnelStatus) -> Void)?
    nonisolated(unsafe) var didAddMoreCredit: (@Sendable () -> Void)?

    init(storePaymentManager: StorePaymentManager, tunnelManager: TunnelManager) {
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager

        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { [weak self] _, tunnelStatus in
                self?.didReceiveTunnelStatus?(tunnelStatus)
            },
            didUpdateDeviceState: { [weak self] _, deviceState, previousDeviceState in
                let isInactive = previousDeviceState.accountData?.isExpired == true
                let isActive = deviceState.accountData?.isExpired == false
                if isInactive && isActive {
                    self?.didAddMoreCredit?()
                }
            }
        )

        let paymentObserver = StorePaymentBlockObserver { [weak self] _, event in
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
        completionHandler: @escaping @Sendable (Result<
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
        completionHandler: @escaping @Sendable (Result<SKProductsResponse, Error>) -> Void
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

        timer.schedule(
            wallDeadline: .now() + accountUpdateTimerInterval,
            repeating: accountUpdateTimerInterval.timeInterval
        )
        timer.activate()
    }

    func stopAccountUpdateTimer() {
        logger.debug(
            "Stop polling account updates."
        )

        accountUpdateTimer?.cancel()
        accountUpdateTimer = nil
    }
}
