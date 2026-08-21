// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Routing
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
