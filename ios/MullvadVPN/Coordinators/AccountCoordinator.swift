//
//  AccountCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-14.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Routing
import UIKit
import StoreKit

enum AccountDismissReason: Equatable, Sendable {
    case none
    case userLoggedOut
    case accountDeletion
}

enum AddedMoreCreditOption: Equatable, Sendable {
    case redeemingVoucher
    case inAppPurchase
}

final class AccountCoordinator: Coordinator, Presentable, Presenting, @unchecked Sendable {
    private let interactor: AccountInteractor
    private var accountController: AccountViewController?

    let navigationController: UINavigationController
    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: (@MainActor (AccountCoordinator, AccountDismissReason) -> Void)?

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
            errorPresenter: PaymentAlertPresenter(alertContext: self)
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
        case .restorePurchasesInfo:
            showRestorePurchasesInfo()
        case .showPurchaseOptions(let details):
            showPurchaseOptions(availableProducts: details.products, accountNumber: details.accountNumber, didRequestPurchase: details.didRequestPurchase)
        case .showFailedToLoadProducts:
            showFailToFetchProducts()
        }
    }
    
    func showPurchaseOptions(availableProducts: [SKProduct], accountNumber: String, didRequestPurchase: @escaping (_ product: SKProduct) -> Void) {
        let localizedString = NSLocalizedString(
            "BUY_CREDIT_BUTTON",
            tableName: "Welcome",
            value: "Add Time",
            comment: ""
        )
        let alert = UIAlertController(title: localizedString, message: nil, preferredStyle: .actionSheet)
        availableProducts.forEach { product in
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

    private func navigateToRedeemVoucher() {
        let coordinator = ProfileVoucherCoordinator(
            navigationController: CustomNavigationController(),
            interactor: RedeemVoucherInteractor(
                tunnelManager: interactor.tunnelManager,
                accountsProxy: interactor.accountsProxy,
                verifyVoucherAsAccount: false
            )
        )
        coordinator.didFinish = { coordinator in
            coordinator.dismiss(animated: true)
        }
        coordinator.didCancel = { coordinator in
            coordinator.dismiss(animated: true)
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

    @MainActor
    private func navigateToDeleteAccount() {
        let coordinator = AccountDeletionCoordinator(
            navigationController: CustomNavigationController(),
            interactor: AccountDeletionInteractor(tunnelManager: interactor.tunnelManager)
        )

        coordinator.start()
        coordinator.didCancel = { accountDeletionCoordinator in
            Task { @MainActor in
                accountDeletionCoordinator.dismiss(animated: true)
            }
        }

        coordinator.didFinish = { @MainActor accountDeletionCoordinator in
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
        let presentation = AlertPresentation(
            id: "account-logout-alert",
            accessibilityIdentifier: .logOutSpinnerAlertView,
            icon: .spinner,
            message: nil,
            buttons: []
        )

        let alertPresenter = AlertPresenter(context: self)

        Task {
            await interactor.logout()
            DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) { [weak self] in
                guard let self else { return }

                alertPresenter.dismissAlert(presentation: presentation, animated: true)
                self.didFinish?(self, .userLoggedOut)
            }
        }

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }

    private func showAccountDeviceInfo() {
        let message = NSLocalizedString(
            "DEVICE_INFO_DIALOG_MESSAGE_PART_1",
            tableName: "Account",
            value: """
            This is the name assigned to the device. Each device logged in on a Mullvad account gets a unique name \
            that helps you identify it when you manage your devices in the app or on the website.
            You can have up to 5 devices logged in on one Mullvad account.
            If you log out, the device and the device name is removed. When \
            you log back in again, the device will get a new name.
            """,
            comment: ""
        )

        let presentation = AlertPresentation(
            id: "account-device-info-alert",
            icon: .info,
            message: message,
            buttons: [AlertAction(
                title: NSLocalizedString(
                    "DEVICE_INFO_DIALOG_OK_ACTION",
                    tableName: "Account",
                    value: "Got it!",
                    comment: ""
                ),
                style: .default
            )]
        )

        let presenter = AlertPresenter(context: self)
        presenter.showAlert(presentation: presentation, animated: true)
    }

    private func showRestorePurchasesInfo() {
        let message = NSLocalizedString(
            "RESTORE_PURCHASES_DIALOG_MESSAGE",
            tableName: "Account",
            value: """
            You can use the "restore purchases" function to check for any in-app payments \
            made via Apple services. If there is a payment that has not been credited, it will \
            add the time to the currently logged in Mullvad account.
            """,
            comment: ""
        )

        let presentation = AlertPresentation(
            id: "account-device-info-alert",
            icon: .info,
            title: NSLocalizedString(
                "RESTORE_PURCHASES_DIALOG_TITLE",
                tableName: "Account",
                value: "If you haven’t received additional VPN time after purchasing",
                comment: ""
            ),
            message: message,
            buttons: [AlertAction(
                title: NSLocalizedString(
                    "RESTORE_PURCHASES_DIALOG_OK_ACTION",
                    tableName: "Account",
                    value: "Got it!",
                    comment: ""
                ),
                style: .default
            )]
        )

        let presenter = AlertPresenter(context: self)
        presenter.showAlert(presentation: presentation, animated: true)
    }
    
    
    func showFailToFetchProducts() {
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
