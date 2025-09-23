//
//  WelcomeCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-28.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import Routing
import StoreKit
import UIKit

final class WelcomeCoordinator: Coordinator, Poppable, Presenting {
    private let navigationController: RootContainerViewController
    private let storePaymentManager: StorePaymentManager
    private let tunnelManager: TunnelManager
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
    }

    func start(animated: Bool) {
        let interactor = WelcomeInteractor(
            tunnelManager: tunnelManager
        )

        interactor.didAddMoreCredit = { [weak self] in
            self?.showSetupAccountCompleted()
        }

        let controller = WelcomeViewController(interactor: interactor)
        controller.delegate = self

        viewController = controller

        navigationController.pushViewController(controller, animated: animated)
    }

    func showSetupAccountCompleted() {
        let coordinator = SetupAccountCompletedCoordinator(navigationController: navigationController)
        coordinator.didFinish = { [weak self] coordinator in
            coordinator.removeFromParent()
            self?.didFinish?()
        }
        addChild(coordinator)
        coordinator.start(animated: true)
    }

    func popFromNavigationStack(animated: Bool, completion: (() -> Void)?) {
        guard let viewController,
            let index = navigationController.viewControllers.firstIndex(of: viewController)
        else {
            completion?()
            return
        }
        navigationController.setViewControllers(
            Array(navigationController.viewControllers[0..<index]),
            animated: animated,
            completion: completion
        )
    }
}

extension WelcomeCoordinator: @preconcurrency WelcomeViewControllerDelegate {
    func didRequestToShowFailToFetchProducts(controller: WelcomeViewController) {
        let message = NSLocalizedString("Failed to connect to App store, please try again later.", comment: "")

        let presentation = AlertPresentation(
            id: "welcome-failed-to-fetch-products-alert",
            icon: .info,
            message: message,
            buttons: [
                AlertAction(
                    title: NSLocalizedString("Got it!", comment: ""),
                    style: .default
                )
            ]
        )

        let presenter = AlertPresenter(context: self)
        presenter.showAlert(presentation: presentation, animated: true)
    }

    func didRequestToShowInfo(controller: WelcomeViewController) {
        let message = [
            NSLocalizedString(
                "This is the name assigned to the device. Each device logged in on a "
                    + "Mullvad account gets a unique name that helps "
                    + "you identify it when you manage your devices in the app or on the website.",
                comment: ""
            ),
            NSLocalizedString(
                "You can have up to 5 devices logged in on one Mullvad account.",
                comment: ""
            ),
            NSLocalizedString(
                "If you log out, the device and the device name is removed. "
                    + "When you log back in again, the device will get a new name.",
                comment: ""
            ),
        ].joinedParagraphs(lineBreaks: 1)

        let presentation = AlertPresentation(
            id: "welcome-device-name-alert",
            icon: .info,
            message: message,
            buttons: [
                AlertAction(
                    title: NSLocalizedString("Got it!", comment: ""),
                    style: .default
                )
            ]
        )

        let presenter = AlertPresenter(context: self)
        presenter.showAlert(presentation: presentation, animated: true)
    }

    func didRequestToViewPurchaseOptions(
        accountNumber: String
    ) {
        let coordinator = InAppPurchaseCoordinator(
            storePaymentManager: storePaymentManager,
            accountNumber: accountNumber,
            paymentAction: .purchase
        )
        coordinator.didFinish = { coordinator in
            coordinator.dismiss(animated: true)
        }
        coordinator.start()
        presentChild(coordinator, animated: true)
    }
}
