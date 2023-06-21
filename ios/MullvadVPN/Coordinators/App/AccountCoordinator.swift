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
}

final class AccountCoordinator: Coordinator, Presentable, Presenting {
    private let interactor: AccountInteractor
    private var accountController: AccountViewController?

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

        let accountController = AccountViewController(interactor: interactor)
        accountController.delegate = self

        navigationController.pushViewController(accountController, animated: animated)
        self.accountController = accountController
    }
}

extension AccountCoordinator: AccountViewControllerDelegate {
    func accountViewController(
        _ controller: AccountViewController,
        didRequestRoutePresentation route: AccountsNavigationRoute
    ) {
        switch route {
        case .redeemVoucher:
            let coordinator = RedeemVoucherCoordinator(
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
                    preferredContentSize: UIMetrics.RedeemVoucher.preferredContentSize,
                    modalPresentationStyle: .custom,
                    transitioningDelegate: FormSheetTransitioningDelegate()
                )
            )
        }
    }

    func accountViewControllerDidFinish(_ controller: AccountViewController) {
        didFinish?(self, .none)
    }

    func accountViewControllerDidLogout(_ controller: AccountViewController) {
        didFinish?(self, .userLoggedOut)
    }
}
