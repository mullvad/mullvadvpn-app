//
//  AccountCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-14.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import UIKit

enum AccountDismissReason: Equatable {
    case none
    case userLoggedOut
}

final class AccountCoordinator: Coordinator, Presentable, Presenting {
    private let interactor: AccountInteractor
    private var accountController: AccountViewController?
    private let alertPresenter = AlertPresenter()

    let navigationController: UINavigationController
    var presentedViewController: UIViewController {
        return navigationController
    }

    var presentationContext: UIViewController {
        return navigationController
    }

    var didFinish: ((AccountCoordinator, AccountDismissReason) -> Void)?

    init(
        navigationController: UINavigationController,
        interactor: AccountInteractor
    ) {
        self.navigationController = navigationController
        self.interactor = interactor
    }

    func start(animated: Bool) {
        navigationController.navigationBar.prefersLargeTitles = true

        let accountController = AccountViewController(interactor: interactor)
        accountController.delegate = self

        navigationController.pushViewController(accountController, animated: animated)
        self.accountController = accountController
    }

    // MARK: - Alerts

    func showLogoutAlert() {
        let alertController = CustomAlertViewController(
            icon: .spinner
        )

        alertPresenter.enqueue(alertController, presentingController: presentationContext) {
            self.interactor.logout {
                DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) { [weak self] in
                    guard let self = self else { return }

                    alertController.dismiss(animated: true) {
                        self.didFinish?(self, .userLoggedOut)
                    }
                }
            }
        }
    }

    func showPaymentErrorAlert(error: StorePaymentManagerError) {
        let alertController = CustomAlertViewController(
            title: NSLocalizedString(
                "CANNOT_COMPLETE_PURCHASE_ALERT_TITLE",
                tableName: "Account",
                value: "Cannot complete the purchase",
                comment: ""
            ),
            message: error.displayErrorDescription
        )

        alertController.addAction(
            title: NSLocalizedString(
                "CANNOT_COMPLETE_PURCHASE_ALERT_OK_ACTION",
                tableName: "Account",
                value: "Got it!",
                comment: ""
            ),
            style: .default
        )

        alertPresenter.enqueue(alertController, presentingController: presentationContext)
    }

    func showRestorePurchasesErrorAlert(error: StorePaymentManagerError) {
        let alertController = CustomAlertViewController(
            title: NSLocalizedString(
                "RESTORE_PURCHASES_FAILURE_ALERT_TITLE",
                tableName: "Account",
                value: "Cannot restore purchases",
                comment: ""
            ),
            message: error.displayErrorDescription
        )

        alertController.addAction(
            title: NSLocalizedString(
                "RESTORE_PURCHASES_FAILURE_ALERT_OK_ACTION",
                tableName: "Account",
                value: "Got it!",
                comment: ""
            ),
            style: .default
        )

        alertPresenter.enqueue(alertController, presentingController: presentationContext)
    }

    func showTimeAddedConfirmationAlert(
        with response: REST.CreateApplePaymentResponse,
        context: REST.CreateApplePaymentResponse.Context
    ) {
        let alertController = CustomAlertViewController(
            title: response.alertTitle(context: context),
            message: response.alertMessage(context: context)
        )

        alertController.addAction(
            title: NSLocalizedString(
                "TIME_ADDED_ALERT_OK_ACTION",
                tableName: "Account",
                value: "Got it!",
                comment: ""
            ),
            style: .default
        )

        alertPresenter.enqueue(alertController, presentingController: presentationContext)
    }

    func showAccountDeviceInfo() {
        let messages = [
            NSLocalizedString(
                "DEVICE_INFO_DIALOG_MESSAGE_PART_1",
                tableName: "Account",
                value: "This is the name assigned to the device. Each device logged in on a Mullvad account gets a unique name that helps you identify it when you manage your devices in the app or on the website.\n\nYou can have up to 5 devices logged in on one Mullvad account.\n\nIf you log out, the device and the device name is removed. When you log back in again, the device will get a new name.",
                comment: ""
            )
        ]

        let alertController = CustomAlertViewController(
            messages: messages,
            icon: .info
        )

        alertController.addAction(
            title: NSLocalizedString(
                "DEVICE_INFO_DIALOG_OK_ACTION",
                tableName: "Account",
                value: "Got it!",
                comment: ""
            ),
            style: .default
        )

        alertPresenter.enqueue(alertController, presentingController: presentationContext)
    }
}

extension AccountCoordinator: AccountViewControllerDelegate {
    func accountViewControllerDidFinish(_ controller: AccountViewController) {
        didFinish?(self, .none)
    }

    func accountViewControllerDidLogout(_ controller: AccountViewController) {
        showLogoutAlert()
    }
}
