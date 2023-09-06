//
//  CreateAccountVoucherCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-03.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import Routing
import UIKit

public class CreateAccountVoucherCoordinator: Coordinator, Presentable {
    private let navigationController: RootContainerViewController
    private let viewController: RedeemVoucherViewController
    private let interactor: CreateAccountVoucherInteractor

    private var accountNumber: String?
    private lazy var accountNumberInputErrorView = {
        VerifiedAccountView {
            self.logout()
        }
    }()

    var didFinish: ((CreateAccountVoucherCoordinator) -> Void)?
    var didCancel: ((CreateAccountVoucherCoordinator) -> Void)?
    var didLogout: ((CreateAccountVoucherCoordinator) -> Void)?

    /**
     Name of notification posted when current account number changes.
     */
    static let didChangePreferredAccountNumber = Notification
        .Name(rawValue: "CreateAccountVoucherCoordinatorDidChangeAccountNumber")

    /**
     User info key passed along with `didChangePreferredAccountNumber` notification that contains string value that
     indicates the new account number.
     */
    static let preferredAccountNumberUserInfoKey = "preferredAccountNumber"

    public var presentedViewController: UIViewController {
        viewController
    }

    init(
        navigationController: RootContainerViewController,
        tunnelManager: TunnelManager,
        accountsProxy: REST.AccountsProxy
    ) {
        self.navigationController = navigationController
        interactor = CreateAccountVoucherInteractor(tunnelManager: tunnelManager, accountsProxy: accountsProxy)
        viewController = RedeemVoucherViewController(interactor: interactor)
    }

    func start() {
        viewController.delegate = self
        interactor.didInputAccountNumber = { [weak self] value in
            guard let self else { return }
            accountNumberInputErrorView.fadeIn()
            accountNumber = value
        }
        navigationController.pushViewController(viewController, animated: true)
    }

    private func logout() {
        let alertController = CustomAlertViewController(icon: .spinner)
        presentedViewController.present(alertController, animated: true, completion: {
            self.interactor.logout { [weak self] in
                alertController.dismiss(animated: true, completion: {
                    self?.accountNumber.flatMap {
                        guard let self else { return }
                        self.didLogout?(self)
                        self.notify(accountNumber: $0)
                    }
                })
            }
        })
    }

    /// Posts `didChangePreferredAccountNumber` notification.
    private func notify(accountNumber: String) {
        NotificationCenter.default.post(
            name: Self.didChangePreferredAccountNumber,
            object: self,
            userInfo: [Self.preferredAccountNumberUserInfoKey: accountNumber]
        )
    }
}

extension CreateAccountVoucherCoordinator: RedeemVoucherViewControllerDelegate {
    func redeemVoucherDidSucceed(_ controller: RedeemVoucherViewController, with response: REST.SubmitVoucherResponse) {
        let coordinator = AddCreditSucceededCoordinator(
            purchaseType: .redeemingVoucher,
            timeAdded: response.timeAdded,
            navigationController: navigationController
        )

        coordinator.didFinish = { [weak self] coordinator in
            coordinator.removeFromParent()
            guard let self else { return }
            didFinish?(self)
        }

        addChild(coordinator)
        coordinator.start()
    }

    func redeemVoucherDidCancel(_ controller: RedeemVoucherViewController) {
        didCancel?(self)
    }

    func viewForInputingAccountNumberAsVoucher(_ controller: RedeemVoucherViewController) -> UIView? {
        accountNumberInputErrorView
    }
}
