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

    var didBuyMoreCredit: ((TimeInterval) -> Void)?

    var didChangeInAppPurchaseState: ((WelcomeInteractor, ProductState) -> Void)?

    deinit {
        accountUpdateTimer?.cancel()
    }

    var viewDidLoad = false {
        didSet {
            guard viewDidLoad else { return }
            requestAccessToStore()
            startAccountUpdateTimer()
            addObservers()
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
            didChangeInAppPurchaseState?(self, .cannotMakePurchases)
        } else {
            let product = StoreSubscription.thirtyDays
            didChangeInAppPurchaseState?(self, .fetching(product))
            _ = storePaymentManager.requestProducts(with: [product]) { [weak self] result in
                guard let self else { return }
                let product = result.value?.products.first
                let productState: ProductState = product.map { .received($0) } ?? .failed
                didChangeInAppPurchaseState?(self, productState)
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

    private func addObservers() {
        let tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] manager, deviceState, previousDeviceState in
                guard case let .loggedIn(oldAccountData, _) = previousDeviceState,
                      case let .loggedIn(newAccountData, _) = deviceState else {
                    return
                }
                guard oldAccountData.isExpired, !newAccountData.isExpired else {
                    return
                }
                self?.didBuyMoreCredit?(newAccountData.expiry - Date())
                AccountFlow.isOnboarding = false
            })
        self.tunnelObserver = tunnelObserver
        tunnelManager.addObserver(tunnelObserver)
    }
}

private extension Date {
    static func - (lhs: Date, rhs: Date) -> TimeInterval {
        return lhs.timeIntervalSinceReferenceDate - rhs.timeIntervalSinceReferenceDate
    }
}
