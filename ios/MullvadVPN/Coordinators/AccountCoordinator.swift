//
//  AccountCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import Routing
import StoreKit
import SwiftUI
import UIKit

enum AccountDismissReason: Equatable, Sendable {
    case none
    case userLoggedOut
    case accountDeletion
}

final class AccountCoordinator: Coordinator, Presentable, Presenting, @unchecked Sendable {
    private let interactor: AccountInteractor
    private let storePaymentManager: StorePaymentManager
    private var accountController: AccountViewController?

    let navigationController: UINavigationController
    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: (@MainActor (AccountCoordinator, AccountDismissReason) -> Void)?

    init(
        navigationController: UINavigationController,
        interactor: AccountInteractor,
        storePaymentManager: StorePaymentManager
    ) {
        self.navigationController = navigationController
        self.interactor = interactor
        self.storePaymentManager = storePaymentManager
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
        case .deviceManagement:
            navigateToDeviceManagement()
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
        case .showFailedToLoadProducts:
            showFailToFetchProducts()
        case .showRestorePurchases:
            didRequestShowInAppPurchase(paymentAction: .restorePurchase)
        case .showPurchaseOptions:
            didRequestShowInAppPurchase(paymentAction: .purchase)
        }
    }

    private func didRequestShowInAppPurchase(
        paymentAction: PaymentAction
    ) {
        guard let accountNumber = interactor.deviceState.accountData?.number else { return }
        let coordinator = InAppPurchaseCoordinator(
            storePaymentManager: storePaymentManager,
            accountNumber: accountNumber,
            paymentAction: paymentAction
        )
        coordinator.didFinish = { coordinator in
            coordinator.dismiss(animated: true)
        }
        coordinator.start()
        presentChild(coordinator, animated: true)
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

    private func navigateToDeviceManagement() {
        guard let accountNumber = interactor.deviceState.accountData?.number,
              let currentDeviceId = interactor.deviceState.deviceData?.identifier else {
            return
        }
        let controller = UIHostingController(
            rootView: DeviceManagementView(
                deviceManaging: DeviceManagementInteractor(
                    accountNumber: accountNumber,
                    currentDeviceId: currentDeviceId,
                    devicesProxy: interactor.deviceProxy
                ),
                style: .normal,
                onError: { title, error in
                    let errorDescription = if case let .network(urlError) = error as? REST.Error {
                        urlError.localizedDescription
                    } else {
                        error.localizedDescription
                    }
                    let presentation = AlertPresentation(
                        id: "device-management-error-alert",
                        title: title,
                        message: errorDescription,
                        buttons: [
                            AlertAction(
                                title: NSLocalizedString(
                                    "ERROR_ALERT_OK_ACTION",
                                    tableName: "DeviceManagement",
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
            )
        )
        controller.title = NSLocalizedString(
            "MANAGE_DEVICES_TITLE",
            tableName: "Manage devices",
            value: "Manage devices",
            comment: ""
        )
        navigationController.pushViewController(controller, animated: true)
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
