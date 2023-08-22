//
//  AddCreditSucceededCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-01.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

final class AddCreditSucceededCoordinator: Coordinator {
    var didFinish: ((AddCreditSucceededCoordinator) -> Void)?

    let paymentType: PurchaseType
    let timeAdded: Int
    let navigationController: RootContainerViewController

    enum PurchaseType {
        case redeemingVoucher, inAppPurchase
    }

    init(purchaseType: PurchaseType, timeAdded: Int, navigationController: RootContainerViewController) {
        self.timeAdded = timeAdded
        self.navigationController = navigationController
        self.paymentType = purchaseType
    }

    func start() {
        let controller =
            AddCreditSucceededViewController(timeAddedComponents: DateComponents(second: timeAdded))
        controller.delegate = self
        self.navigationController.pushViewController(controller, animated: true)
    }
}

extension AddCreditSucceededCoordinator: AddCreditSucceededViewControllerDelegate {
    func addCreditSucceededViewControllerDidFinish() {
        let coordinator = SetupAccountCompletedCoordinator(navigationController: navigationController)
        coordinator.didFinish = { [weak self] coordinator in
            coordinator.removeFromParent()
            guard let self else { return }
            didFinish?(self)
        }
        addChild(coordinator)
        coordinator.start(animated: true)
    }

    func titleForAction() -> String {
        NSLocalizedString(
            "ADDED_TIME_SUCCESS_DISMISS_BUTTON",
            tableName: "Welcome",
            value: "Next",
            comment: ""
        )
    }

    func header() -> String {
        switch paymentType {
        case .inAppPurchase:
            return NSLocalizedString(
                "IN_APP_PURCHASE_SUCCESS_TITLE",
                tableName: "Welcome",
                value: "Time was successfully added.",
                comment: ""
            )
        case .redeemingVoucher:
            return NSLocalizedString(
                "REDEEM_VOUCHER_SUCCESS_TITLE",
                tableName: "Welcome",
                value: "Voucher was successfully redeemed.",
                comment: ""
            )
        }
    }
}
