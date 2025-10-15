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
                transitioningDelegate: FormSheetTransitioningDelegate(
                    options: FormSheetPresentationOptions(
                        useFullScreenPresentationInCompactWidth: false,
                        adjustViewWhenKeyboardAppears: true
                    ))
            )
        )
    }

    private func navigateToDeviceManagement() {
        guard let accountNumber = interactor.deviceState.accountData?.number,
            let currentDeviceId = interactor.deviceState.deviceData?.identifier
        else {
            return
        }
        let controller = UIHostingController(
            rootView: DeviceManagementView(
                deviceManaging: DeviceManagementInteractor(
                    accountNumber: accountNumber,
                    currentDeviceId: currentDeviceId,
                    devicesProxy: interactor.deviceProxy
                ),
                style: .deviceManagement,
                onError: { [weak self] title, error in
                    self?.presentError(
                        "device-management-error-alert",
                        title: title,
                        message: error.localizedDescription
                    )
                }
            )
        )
        controller.title = NSLocalizedString("Manage devices", comment: "")
        let doneButton = UIBarButtonItem(
            systemItem: .done,
            primaryAction: UIAction(handler: { _ in
                controller.dismiss(animated: true)
            })
        )
        controller.navigationItem.rightBarButtonItem = doneButton
        let subNavigationController = CustomNavigationController(rootViewController: controller)
        subNavigationController.navigationItem.largeTitleDisplayMode = .always
        subNavigationController.navigationBar.prefersLargeTitles = true
        navigationController.present(subNavigationController, animated: true)
    }

    private func presentError(_ id: String, title: String, message: String) {
        let presentation = AlertPresentation(
            id: id,
            title: title,
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

    @MainActor
    private func navigateToDeleteAccount() {
        let coordinator = AccountDeletionCoordinator(
            navigationController: CustomNavigationController(),
            tunnelManager: interactor.tunnelManager
        )

        coordinator.start()
        coordinator.didConclude = { accountDeletionCoordinator, success in
            Task { @MainActor in
                accountDeletionCoordinator.dismiss(
                    animated: true,
                    completion: {
                        if success { self.didFinish?(self, .userLoggedOut) }
                    }
                )
            }
        }

        presentChild(
            coordinator,
            animated: true,
            configuration: ModalPresentationConfiguration(
                preferredContentSize: UIMetrics.AccountDeletion.preferredContentSize,
                modalPresentationStyle: .custom,
                transitioningDelegate: FormSheetTransitioningDelegate(
                    options: FormSheetPresentationOptions(
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
            """
            You can use the "restore purchases" function to check for any in-app payments \
            made via Apple services. If there is a payment that has not been credited, it will \
            add the time to the currently logged in Mullvad account.
            """,
            comment: ""
        )

        let presentation = AlertPresentation(
            id: "account-device-info-alert",
            icon: .info,
            title: NSLocalizedString("If you haven’t received additional VPN time after purchasing", comment: ""),
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

    func showFailToFetchProducts() {
        let message = NSLocalizedString(
            "Failed to load products, please try again",
            comment: ""
        )

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
}
