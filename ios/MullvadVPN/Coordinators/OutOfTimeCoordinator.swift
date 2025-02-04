//
//  OutOfTimeCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 10/03/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Routing
import StoreKit
import UIKit

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

    func outOfTimeViewControllerDidRequestShowPurchaseOptions(
        _ controller: OutOfTimeViewController,
        products: [SKProduct],
        didRequestPurchase: @escaping (SKProduct) -> Void
    ) {
        let alert = UIAlertController.showInAppPurchaseAlert(products: products, didRequestPurchase: didRequestPurchase)
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
