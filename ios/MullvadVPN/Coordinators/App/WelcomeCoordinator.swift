//
//  WelcomeCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-28.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import UIKit

final class WelcomeCoordinator: Coordinator, Presentable {
    private let navigationController: RootContainerViewController
    private let storePaymentManager: StorePaymentManager
    private let tunnelManager: TunnelManager
    private var viewController: WelcomeViewController?

    var didFinishPayment: ((WelcomeCoordinator) -> Void)?

    var presentedViewController: UIViewController {
        navigationController
    }

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
        guard case let .loggedIn(storedAccountData, storedDeviceData) = tunnelManager.deviceState else {
            return
        }
        let interactor = WelcomeInteractor(
            deviceData: storedDeviceData,
            accountData: storedAccountData,
            tunnelManager: tunnelManager
        )

        let controller = WelcomeViewController(interactor: interactor)
        controller.delegate = self

        viewController = controller

        navigationController.pushViewController(controller, animated: animated)
    }

    func popFromNavigationStack(animated: Bool, completion: @escaping () -> Void) {
        guard let viewController,
              let index = navigationController.viewControllers.firstIndex(of: viewController)
        else {
            completion()
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
        let message = """
        This is the name assigned to the device. Each device logged in on a \
        Mullvad account gets a unique name that helps \
        you identify it when you manage your devices in the app or on the website.

        You can have up to 5 devices logged in on one Mullvad account.

        If you log out, the device and the device name is removed. \
        When you log back in again, the device will get a new name.
        """
        let alertController = CustomAlertViewController(
            message: NSLocalizedString(
                "WELCOME_DEVICE_CONCEPET_TEXT_DIALOG",
                tableName: "Welcome",
                value: message,
                comment: ""
            ),
            icon: .info
        )

        alertController.addAction(
            title: NSLocalizedString(
                "WELCOME_DEVICE_NAME_DIALOG_OK_ACTION",
                tableName: "Welcome",
                value: "Got it!",
                comment: ""
            ),
            style: .default
        )
        presentedViewController.present(alertController, animated: true)
    }

    func didRequestToPurchaseCredit(controller: WelcomeViewController) {
        // TODO: In-app purchase
    }

    func didRequestToRedeemVoucher(controller: WelcomeViewController) {
        let coordinator = AccountRedeemingVoucherCoordinator(
            navigationController: navigationController,
            interactor: RedeemVoucherInteractor(tunnelManager: tunnelManager)
        )

        coordinator.didCancel = { [weak self] coordinator in
            guard let self else { return }
            navigationController.popViewController(animated: true)
            coordinator.removeFromParent()
        }

        coordinator.didFinish = { [weak self] coordinator in
            guard let self else { return }
            coordinator.removeFromParent()
            didFinishPayment?(self)
        }

        addChild(coordinator)

        coordinator.start()
    }

    func didUpdateDeviceState(deviceState: DeviceState) {
        if deviceState.accountData?.isExpired == false {
            didFinishPayment?(self)
        }
    }
}
