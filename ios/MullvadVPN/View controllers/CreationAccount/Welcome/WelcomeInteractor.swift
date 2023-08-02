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

    /// Interval used for periodic pulling account updates.
    private let logger = Logger(label: "\(WelcomeInteractor.self)")

    var didChangeInAppPurchaseState: ((WelcomeInteractor, ProductState) -> Void)?
    var productState: ProductState = .none {
        didSet {
            didChangeInAppPurchaseState?(self, productState)
        }
    }

    var viewDidLoad = false {
        didSet {
            guard viewDidLoad else { return }
            requestAccessToStore()
        }
    }

    var accountNumber: String {
        tunnelManager.deviceState.accountData?.number ?? ""
    }

    var viewModel: WelcomeViewModel {
        WelcomeViewModel(
            deviceName: tunnelManager.deviceState.deviceData?.capitalizedName ?? "",
            accountNumber: accountNumber.formattedAccountNumber
        )
    }

    init(storePaymentManager: StorePaymentManager, tunnelManager: TunnelManager) {
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager
    }

    private func requestAccessToStore() {
        if !StorePaymentManager.canMakePayments {
            productState = .cannotMakePurchases
        } else {
            let product = StoreSubscription.thirtyDays
            didChangeInAppPurchaseState?(self, .fetching(product))
            _ = storePaymentManager.requestProducts(with: [product]) { [weak self] result in
                guard let self else { return }
                let productState: ProductState = result.value?.products.first
                    .map { .received($0) } ?? .failed
                self.productState = productState
            }
        }
    }
}
