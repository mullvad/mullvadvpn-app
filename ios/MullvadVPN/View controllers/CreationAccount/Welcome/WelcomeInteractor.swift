//
//  WelcomeInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-29.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import StoreKit

final class WelcomeInteractor {
    private let storePaymentManager: StorePaymentManager
    private let tunnelManager: TunnelManager

    /// Interval used for periodic polling account updates.
    private let accountUpdateTimerInterval: Duration = .minutes(1)
    private var accountUpdateTimer: DispatchSourceTimer?

    private let logger = Logger(label: "\(WelcomeInteractor.self)")
    private var tunnelObserver: TunnelObserver?
    private(set) var product: SKProduct?

    var didChangeInAppPurchaseState: ((ProductState) -> Void)?
    var didAddMoreCredit: (() -> Void)?

    var viewDidLoad = false {
        didSet {
            guard viewDidLoad else { return }
            requestAccessToStore()
        }
    }

    var viewWillAppear = false {
        didSet {
            guard viewWillAppear else { return }
            startAccountUpdateTimer()
        }
    }

    var viewDidDisappear = false {
        didSet {
            guard viewDidDisappear else { return }
            stopAccountUpdateTimer()
        }
    }

    var accountNumber: String {
        tunnelManager.deviceState.accountData?.number ?? ""
    }

    var viewModel: WelcomeViewModel {
        WelcomeViewModel(
            deviceName: tunnelManager.deviceState.deviceData?.capitalizedName ?? "",
            accountNumber: tunnelManager.deviceState.accountData?.number.formattedAccountNumber ?? ""
        )
    }

    init(
        storePaymentManager: StorePaymentManager,
        tunnelManager: TunnelManager
    ) {
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager
        let tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] _, deviceState, previousDeviceState in
                let isInactive = previousDeviceState.accountData?.isExpired == true
                let isActive = deviceState.accountData?.isExpired == false
                if isInactive && isActive {
                    self?.didAddMoreCredit?()
                }
            })

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver
    }

    private func requestAccessToStore() {
        if !StorePaymentManager.canMakePayments {
            didChangeInAppPurchaseState?(.cannotMakePurchases)
        } else {
            let product = StoreSubscription.thirtyDays
            didChangeInAppPurchaseState?(.fetching(product))
            _ = storePaymentManager.requestProducts(with: [product]) { [weak self] result in
                guard let self else { return }
                let product = result.value?.products.first
                let productState: ProductState = product.map { .received($0) } ?? .failed
                didChangeInAppPurchaseState?(productState)
                self.product = product
            }
        }
    }

    private func startAccountUpdateTimer() {
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

    private func stopAccountUpdateTimer() {
        logger.debug(
            "Stop polling account updates."
        )

        accountUpdateTimer?.cancel()
        accountUpdateTimer = nil
    }
}
