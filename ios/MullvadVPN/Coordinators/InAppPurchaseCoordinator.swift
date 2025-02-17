//
//  InAppPurchaseCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Routing
import StoreKit
import UIKit

enum PaymentAction {
    case purchase
    case restorePurchase
}

final class InAppPurchaseCoordinator: Coordinator, Presentable, Presenting {
    private var controller: InAppPurchaseViewController?
    private let storePaymentManager: StorePaymentManager
    private let accountNumber: String
    private let paymentAction: PaymentAction

    var didFinish: ((InAppPurchaseCoordinator) -> Void)?

    var presentedViewController: UIViewController {
        return controller!
    }

    init(storePaymentManager: StorePaymentManager, accountNumber: String, paymentAction: PaymentAction) {
        self.storePaymentManager = storePaymentManager
        self.accountNumber = accountNumber
        self.paymentAction = paymentAction
    }

    func dismiss() {
        didFinish?(self)
    }

    func start() {
        controller = InAppPurchaseViewController(
            storePaymentManager: storePaymentManager,
            accountNumber: accountNumber,
            errorPresenter: PaymentAlertPresenter(alertContext: self),
            paymentAction: paymentAction
        )
        controller?.didFinish = dismiss
    }
}
