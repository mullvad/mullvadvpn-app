//
//  WelcomeInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-29.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

class WelcomeInteractor {
    private let storePaymentManager: StorePaymentManager
    private let tunnelManager: TunnelManager

    var didChangeInAppPurchaseState: ((WelcomeInteractor, ProductState) -> Void)?

    var viewDidLoad = false {
        didSet {
            guard viewDidLoad else { return }
            requestAccessToStore()
        }
    }

    var viewModel: WelcomeViewModel {
        WelcomeViewModel(
            deviceName: tunnelManager.deviceState.deviceData?.capitalizedName ?? "",
            accountNumber: tunnelManager.deviceState.accountData?.number.formattedAccountNumber ?? ""
        )
    }

    init(storePaymentManager: StorePaymentManager, tunnelManager: TunnelManager) {
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
                let productState: ProductState = result.value?.products.first
                    .map { .received($0) } ?? .failed
                self.didChangeInAppPurchaseState?(self, productState)
            }
        }
    }
}
