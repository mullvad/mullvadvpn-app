//
//  WelcomeCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-28.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import Routing
import StoreKit
import UIKit

final class WelcomeCoordinator: Coordinator, Poppable, Presenting {
    private let navigationController: RootContainerViewController
    private let storePaymentManager: StorePaymentManager
    private let tunnelManager: TunnelManager
    private let inAppPurchaseInteractor: InAppPurchaseInteractor
    private let accountsProxy: RESTAccountHandling

    private var viewController: WelcomeViewController?

    var didFinish: (() -> Void)?
    var didLogout: ((String) -> Void)?

    var presentedViewController: UIViewController {
        navigationController
    }

    init(
        navigationController: RootContainerViewController,
        storePaymentManager: StorePaymentManager,
        tunnelManager: TunnelManager,
        accountsProxy: RESTAccountHandling
    ) {
        self.navigationController = navigationController
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager
        self.accountsProxy = accountsProxy
        self.inAppPurchaseInteractor = InAppPurchaseInteractor(storePaymentManager: storePaymentManager)
    }

    func start(animated: Bool) {
        let interactor = WelcomeInteractor(
            storePaymentManager: storePaymentManager,
            tunnelManager: tunnelManager
        )

        interactor.didAddMoreCredit = { [weak self] in
            guard let self else { return }
            let coordinator = SetupAccountCompletedCoordinator(navigationController: navigationController)
            coordinator.didFinish = { [weak self] coordinator in
                coordinator.removeFromParent()
                self?.didFinish?()
            }
            addChild(coordinator)
            coordinator.start(animated: true)
        }

        let controller = WelcomeViewController(interactor: interactor)
        controller.delegate = self

        viewController = controller

        navigationController.pushViewController(controller, animated: animated)
    }

    func popFromNavigationStack(animated: Bool, completion: (() -> Void)?) {
        guard let viewController,
              let index = navigationController.viewControllers.firstIndex(of: viewController)
        else {
            completion?()
            return
        }
        navigationController.setViewControllers(
            Array(navigationController.viewControllers[0 ..< index]),
            animated: animated,
            completion: completion
        )
    }
}

extension WelcomeCoordinator: WelcomeViewControllerDelegate {
    func didRequestToShowInfo(controller: WelcomeViewController) {
        let message = NSLocalizedString(
            "WELCOME_DEVICE_CONCEPT_TEXT_DIALOG",
            tableName: "Welcome",
            value:
            """
            This is the name assigned to the device. Each device logged in on a \
            Mullvad account gets a unique name that helps \
            you identify it when you manage your devices in the app or on the website.

            You can have up to 5 devices logged in on one Mullvad account.

            If you log out, the device and the device name is removed. \
            When you log back in again, the device will get a new name.
            """,
            comment: ""
        )

        let presentation = AlertPresentation(
            id: "welcome-device-name-alert",
            icon: .info,
            message: message,
            buttons: [
                AlertAction(
                    title: NSLocalizedString(
                        "WELCOME_DEVICE_NAME_DIALOG_OK_ACTION",
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

    func didRequestToPurchaseCredit(controller: WelcomeViewController, accountNumber: String, product: SKProduct) {
        navigationController.enableHeaderBarButtons(false)

        let coordinator = InAppPurchaseCoordinator(
            navigationController: navigationController,
            interactor: inAppPurchaseInteractor
        )

        inAppPurchaseInteractor.viewControllerDelegate = viewController

        coordinator.didFinish = { [weak self] coordinator in
            guard let self else { return }
            navigationController.enableHeaderBarButtons(true)
            coordinator.removeFromParent()
            didFinish?()
        }

        coordinator.didCancel = { [weak self] coordinator in
            self?.navigationController.enableHeaderBarButtons(true)
            coordinator.removeFromParent()
        }

        addChild(coordinator)

        coordinator.start(accountNumber: accountNumber, product: product)
    }

    func didRequestToRedeemVoucher(controller: WelcomeViewController) {
        let coordinator = CreateAccountVoucherCoordinator(
            navigationController: navigationController,
            interactor: RedeemVoucherInteractor(
                tunnelManager: tunnelManager,
                accountsProxy: accountsProxy,
                verifyVoucherAsAccount: true
            )
        )

        coordinator.didCancel = { [weak self] coordinator in
            guard let self = self else { return }
            navigationController.popViewController(animated: true)
            coordinator.removeFromParent()
        }

        coordinator.didFinish = { [weak self] coordinator in
            guard let self else { return }
            coordinator.removeFromParent()
            didFinish?()
        }

        coordinator.didLogout = { [weak self] coordinator, accountNumber in
            guard let self else { return }
            coordinator.removeFromParent()
            didLogout?(accountNumber)
        }

        addChild(coordinator)

        coordinator.start()
    }
}
