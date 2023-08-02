//
//  InAppPurchaseCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-21.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
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
                self.navigateToAddCreditSucceeded(value.serverResponse.timeAdded)
            case let .failure(failure):
                self.present(failure.error)
            }
        }
    }

    private func navigateToAddCreditSucceeded(_ timeAdded: TimeInterval) {
        let coordinator = AddCreditSucceededCoordinator(
            timeAdded: Int(timeAdded),
            navigationController: self.navigationController
        )

        coordinator.didFinish = { coordinator in
            coordinator.removeFromParent()
            self.didFinish?(self)
        }

        addChild(coordinator)
        coordinator.start()
    }

    private func present(_ error: Error) {
        let alertController = CustomAlertViewController(
            message: error.localizedDescription,
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
        presentedViewController.present(alertController, animated: true)
    }
}
