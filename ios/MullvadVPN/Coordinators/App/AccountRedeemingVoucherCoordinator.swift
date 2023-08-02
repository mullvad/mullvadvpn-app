//
//  AccountRedeemingVoucherCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-03.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import UIKit

class AccountRedeemingVoucherCoordinator: Coordinator, Presentable {
    private let navigationController: RootContainerViewController
    private let viewController: RedeemVoucherViewController

    var didFinish: ((AccountRedeemingVoucherCoordinator) -> Void)?
    var didCancel: ((AccountRedeemingVoucherCoordinator) -> Void)?

    var presentedViewController: UIViewController {
        viewController
    }

    init(
        navigationController: RootContainerViewController,
        interactor: RedeemVoucherInteractor
    ) {
        self.navigationController = navigationController
        viewController = RedeemVoucherViewController(interactor: interactor)
    }

    func start() {
        viewController.delegate = self
        navigationController.pushViewController(viewController, animated: true)
    }
}

extension AccountRedeemingVoucherCoordinator: RedeemVoucherViewControllerDelegate {
    func redeemVoucherDidSucceed(_ controller: RedeemVoucherViewController, with response: REST.SubmitVoucherResponse) {
        let coordinator = AddCreditSucceededCoordinator(
            timeAdded: response.timeAdded,
            navigationController: navigationController
        )

        coordinator.didFinish = { [self] coordinator in
            coordinator.removeFromParent()
            didFinish?(self)
        }

        addChild(coordinator)
        coordinator.start()
    }

    func redeemVoucherDidCancel(_ controller: RedeemVoucherViewController) {
        didCancel?(self)
    }
}
