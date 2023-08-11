//
//  WelcomeInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-29.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import StoreKit

class WelcomeInteractor {
    private let storePaymentManager: StorePaymentManager
    private let tunnelManager: TunnelManager

    /// Interval used for periodic polling account updates.
    private let accountUpdateTimerInterval: TimeInterval = 30
    private var accountUpdateTimer: DispatchSourceTimer?

    private let logger = Logger(label: "\(WelcomeInteractor.self)")
    private var tunnelObserver: TunnelObserver?

    var didBuyMoreCredit: (() -> Void)?

    var didChangeInAppPurchaseState: ((ProductState) -> Void)?

    deinit {
        accountUpdateTimer?.cancel()
    }

    var viewDidLoad = false {
        didSet {
            guard viewDidLoad else { return }
            requestAccessToStore()
            startAccountUpdateTimer()
        }
    }

    var accountNumber: String {
        tunnelManager.deviceState.accountData?.number ?? ""
    }

    private(set) var product: SKProduct? = nil

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

        timer.schedule(wallDeadline: .now() + accountUpdateTimerInterval, repeating: accountUpdateTimerInterval)
        timer.activate()
    }
}
