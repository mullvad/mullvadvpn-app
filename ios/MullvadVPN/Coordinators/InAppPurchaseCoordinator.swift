//
//  InAppPurchaseCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-21.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Routing
import StoreKit
import UIKit

class InAppPurchaseCoordinator: Coordinator, Presentable {
    private let navigationController: RootContainerViewController
    private let interactor: InAppPurchaseInteractor

    var didFinish: ((InAppPurchaseCoordinator) -> Void)?
    var didCancel: ((InAppPurchaseCoordinator) -> Void)?

    var presentedViewController: UIViewController {
        navigationController
    }

    init(navigationController: RootContainerViewController, interactor: InAppPurchaseInteractor) {
        self.navigationController = navigationController
        self.interactor = interactor
    }

    func start(accountNumber: String, product: SKProduct) {
        interactor.purchase(accountNumber: accountNumber, product: product)
        interactor.didFinishPayment = { [weak self] interactor, paymentEvent in
            guard let self else { return }
            switch paymentEvent {
            case let .finished(value):
                let coordinator = AddCreditSucceededCoordinator(
                    purchaseType: .inAppPurchase,
                    timeAdded: Int(value.serverResponse.timeAdded),
                    navigationController: navigationController
                )

                coordinator.didFinish = { [weak self] coordinator in
                    coordinator.removeFromParent()
                    guard let self else { return }
                    didFinish?(self)
                }

                addChild(coordinator)
                coordinator.start()

            case let .failure(failure):
                let alertController = AlertViewController(
                    message: failure.error.localizedDescription,
                    icon: .alert
                )

                alertController.addAction(
                    title: NSLocalizedString(
                        "IN_APP_PURCHASE_ERROR_DIALOG_OK_ACTION",
                        tableName: "Welcome",
                        value: "Got it!",
                        comment: ""
                    ),
                    style: .default
                )
                presentedViewController.present(alertController, animated: true) {
                    self.didCancel?(self)
                }
            }
        }
    }
}
