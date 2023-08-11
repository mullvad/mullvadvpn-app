//
//  AccountCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-14.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum AccountDismissReason: Equatable {
    case none
    case userLoggedOut
    case accountDeletion
}

enum AddedMoreCreditOption: Equatable {
    case redeemingVoucher
    case inAppPurchase
}

final class AccountCoordinator: Coordinator, Presentable, Presenting {
    private let interactor: AccountInteractor
    private var accountController: AccountViewController?
    private let alertPresenter = AlertPresenter()

    let navigationController: UINavigationController
    var presentedViewController: UIViewController {
        navigationController
    }

    var presentationContext: UIViewController {
        navigationController
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

        let accountController = AccountViewController(
            interactor: interactor,
            errorPresenter: PaymentAlertPresenter(
                presentationController: presentationContext,
                alertPresenter: alertPresenter
            )
        )

        accountController.actionHandler = handleViewControllerAction

        navigationController.pushViewController(accountController, animated: animated)
        self.accountController = accountController
    }

    private func handleViewControllerAction(_ action: AccountViewControllerAction) {
        switch action {
        case .deviceInfo:
            showAccountDeviceInfo()
        case .finish:
            didFinish?(self, .none)
        case .logOut:
            logOut()
        case .navigateToVoucher:
            navigateToRedeemVoucher()
        case .navigateToDeleteAccount:
            navigateToDeleteAccount()
        }
    }

    private func navigateToRedeemVoucher() {
        let coordinator = SettingsRedeemVoucherCoordinator(
            navigationController: CustomNavigationController(),
            interactor: RedeemVoucherInteractor(tunnelManager: interactor.tunnelManager)
        )
        coordinator.didFinish = { redeemVoucherCoordinator in
            redeemVoucherCoordinator.dismiss(animated: true)
        }
        coordinator.didCancel = { redeemVoucherCoordinator in
            redeemVoucherCoordinator.dismiss(animated: true)
        }

        coordinator.start()
        presentChild(
            coordinator,
            animated: true,
            configuration: ModalPresentationConfiguration(
                preferredContentSize: UIMetrics.SettingsRedeemVoucher.preferredContentSize,
                modalPresentationStyle: .custom,
                transitioningDelegate: FormSheetTransitioningDelegate(options: FormSheetPresentationOptions(
                    useFullScreenPresentationInCompactWidth: false,
                    adjustViewWhenKeyboardAppears: true
                ))
            )
        )
    }

    private func navigateToDeleteAccount() {
        let coordinator = AccountDeletionCoordinator(
            navigationController: CustomNavigationController(),
            interactor: AccountDeletionInteractor(tunnelManager: interactor.tunnelManager)
        )

        coordinator.start()
        coordinator.didCancel = { accountDeletionCoordinator in
            accountDeletionCoordinator.dismiss(animated: true)
        }

        coordinator.didFinish = { accountDeletionCoordinator in
            accountDeletionCoordinator.dismiss(animated: true) {
                self.didFinish?(self, .userLoggedOut)
            }
        }

        presentChild(
            coordinator,
            animated: true,
            configuration: ModalPresentationConfiguration(
                preferredContentSize: UIMetrics.AccountDeletion.preferredContentSize,
                modalPresentationStyle: .custom,
                transitioningDelegate: FormSheetTransitioningDelegate(options: FormSheetPresentationOptions(
                    useFullScreenPresentationInCompactWidth: true,
                    adjustViewWhenKeyboardAppears: false
                ))
            )
        )
    }

    // MARK: - Alerts

    private func logOut() {
        let alertController = CustomAlertViewController(icon: .spinner)

        alertPresenter.enqueue(alertController, presentingController: presentationContext) {
            self.interactor.logout {
                DispatchQueue.main.asyncAfter(deadline: .now() + DispatchTimeInterval.seconds(1)) { [weak self] in
                    guard let self else { return }

                    alertController.dismiss(animated: true) {
                        self.didFinish?(self, .userLoggedOut)
                    }
                }
            }
        }
    }

    private func showAccountDeviceInfo() {
        let message = NSLocalizedString(
            "DEVICE_INFO_DIALOG_MESSAGE_PART_1",
            tableName: "Account",
            value: """
            This is the name assigned to the device. Each device logged in on a Mullvad account gets a unique name that helps you identify it when you manage your devices in the app or on the website.

            You can have up to 5 devices logged in on one Mullvad account.

            If you log out, the device and the device name is removed. When you log back in again, the device will get a new name.
            """,
            comment: ""
        )

        let alertController = CustomAlertViewController(
            message: message,
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
