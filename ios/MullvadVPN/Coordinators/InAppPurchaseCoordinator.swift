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

class InAppPurchaseCoordinator: Coordinator, Presenting, Presentable {
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
        interactor.didFinishPayment = { [weak self] _, paymentEvent in
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
                let presentation = AlertPresentation(
                    id: "in-app-purchase-error-alert",
                    icon: .alert,
                    message: failure.error.localizedDescription,
                    buttons: [
                        AlertAction(
                            title: NSLocalizedString(
                                "IN_APP_PURCHASE_ERROR_DIALOG_OK_ACTION",
                                tableName: "Welcome",
                                value: "Got it!",
                                comment: ""
                            ),
                            style: .default,
                            handler: { [weak self] in
                                guard let self = self else { return }
                                self.didCancel?(self)
                            }
                        ),
                    ]
                )

                let presenter = AlertPresenter(context: self)
                presenter.showAlert(presentation: presentation, animated: true)
            }
        }
    }
}
