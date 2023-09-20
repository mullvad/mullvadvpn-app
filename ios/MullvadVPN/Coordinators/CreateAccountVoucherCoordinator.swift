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

public class CreateAccountVoucherCoordinator: Coordinator {
    private let navigationController: RootContainerViewController
    private let viewController: RedeemVoucherViewController
    private let interactor: RedeemVoucherInteractor

    var didFinish: ((CreateAccountVoucherCoordinator) -> Void)?
    var didCancel: ((CreateAccountVoucherCoordinator) -> Void)?
    var didLogout: ((CreateAccountVoucherCoordinator, String) -> Void)?

    init(
        navigationController: RootContainerViewController,
        interactor: RedeemVoucherInteractor
    ) {
        self.navigationController = navigationController
        self.interactor = interactor

        var layoutMargins = navigationController.view.layoutMargins.toDirectionalInsets
        layoutMargins.top += UIMetrics.contentLayoutMargins.top
        layoutMargins.bottom += UIMetrics.contentLayoutMargins.bottom

        viewController = RedeemVoucherViewController(
            configuration: RedeemVoucherViewConfiguration(
                adjustViewWhenKeyboardAppears: true,
                shouldUseCompactStyle: false,
                layoutMargins: layoutMargins
            ),
            interactor: interactor
        )
    }

    func start() {
        interactor.didLogout = { [weak self] accountNumber in
            guard let self else { return }
            didLogout?(self, accountNumber)
        }
        viewController.delegate = self
        navigationController.pushViewController(viewController, animated: true)
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
}
