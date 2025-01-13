//
//  OutOfTimeCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 10/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Routing
import UIKit
import StoreKit

class OutOfTimeCoordinator: Coordinator, Presenting, @preconcurrency OutOfTimeViewControllerDelegate, Poppable {
    let navigationController: RootContainerViewController
    let storePaymentManager: StorePaymentManager
    let tunnelManager: TunnelManager

    nonisolated(unsafe) var didFinishPayment: (@Sendable (OutOfTimeCoordinator) -> Void)?

    var presentedViewController: UIViewController {
        navigationController
    }

    private(set) var isMakingPayment = false
    private var viewController: OutOfTimeViewController?

    init(
        navigationController: RootContainerViewController,
        storePaymentManager: StorePaymentManager,
        tunnelManager: TunnelManager
    ) {
        self.navigationController = navigationController
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager
    }

    func start(animated: Bool) {
        let interactor = OutOfTimeInteractor(
            storePaymentManager: storePaymentManager,
            tunnelManager: tunnelManager
        )

        interactor.didAddMoreCredit = { [weak self] in
            guard let self else { return }
            didFinishPayment?(self)
        }

        let controller = OutOfTimeViewController(
            interactor: interactor,
            errorPresenter: PaymentAlertPresenter(alertContext: self)
        )

        controller.delegate = self

        viewController = controller

        navigationController.pushViewController(controller, animated: animated)
    }

    func popFromNavigationStack(animated: Bool, completion: (() -> Void)?) {
        guard let viewController else {
            completion?()
            return
        }

        let viewControllers = navigationController.viewControllers.filter { $0 != viewController }

        navigationController.setViewControllers(
            viewControllers,
            animated: animated,
            completion: completion
        )
    }

    // MARK: - OutOfTimeViewControllerDelegate

    func outOfTimeViewControllerDidBeginPayment(_ controller: OutOfTimeViewController) {
        isMakingPayment = true
    }

    func outOfTimeViewControllerDidEndPayment(_ controller: OutOfTimeViewController) {
        isMakingPayment = false

        didFinishPayment?(self)
    }
    
    func outOfTimeViewControllerDidRequestShowPurchaseOptions(_ controller: OutOfTimeViewController, products: [SKProduct], didRequestPurchase: @escaping (SKProduct) -> Void) {
        let localizedString = NSLocalizedString(
            "BUY_CREDIT_BUTTON",
            tableName: "Welcome",
            value: "Add Time",
            comment: ""
        )
        let alert = UIAlertController(title: localizedString, message: nil, preferredStyle: .actionSheet)
        products.forEach { product in
            guard let localizedTitle = product.customLocalizedTitle else {
                return
            }
            let action = UIAlertAction(title: localizedTitle, style: .default, handler: { _ in
                alert.dismiss(animated: true, completion: {
                    didRequestPurchase(product)
                })
            })
            action.accessibilityIdentifier = "\(AccessibilityIdentifier.purchaseButton.asString)_\(product.productIdentifier)"
            alert.addAction(action)
        }
        let cancelAction = UIAlertAction(title: NSLocalizedString(
            "PRODUCT_LIST_CANCEL_BUTTON",
            tableName: "Welcome",
            value: "Cancel",
            comment: ""
        ), style: .cancel)
        cancelAction.accessibilityIdentifier = AccessibilityIdentifier.cancelPurchaseListButton.asString
        alert.addAction(cancelAction)
        presentationContext.present(alert, animated: true)
    }
    
    func outOfTimeViewControllerDidFailToFetchProducts(_ controller: OutOfTimeViewController) {
        let message = NSLocalizedString(
            "WELCOME_FAILED_TO_FETCH_PRODUCTS_DIALOG",
            tableName: "Welcome",
            value:
            """
            Failed to connect to App store, please try again later.
            """,
            comment: ""
        )

        let presentation = AlertPresentation(
            id: "welcome-failed-to-fetch-products-alert",
            icon: .info,
            message: message,
            buttons: [
                AlertAction(
                    title: NSLocalizedString(
                        "WELCOME_FAILED_TO_FETCH_PRODUCTS_OK_ACTION",
                        tableName: "Welcome",
                        value: "Got it!",
                        comment: ""
                    ),
                    style: .default
                ),
            ]
        )

        let presenter = AlertPresenter(context: self)
        presenter.showAlert(presentation: presentation, animated: true)
    }
}
