//
//  RedeemVoucherCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import UIKit

final class RedeemVoucherCoordinator: Coordinator, Presentable, Presenting {
    private let navigationController: UINavigationController
    private let viewController: RedeemVoucherViewController
    var didFinish: ((Coordinator) -> Void)?
    var didCancel: ((Coordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        interactor: RedeemVoucherInteractor
    ) {
        self.navigationController = navigationController
        viewController = RedeemVoucherViewController(interactor: interactor)
    }

    var presentedViewController: UIViewController {
        return navigationController
    }

    var presentationContext: UIViewController {
        return navigationController
    }

    func start() {
        navigationController.navigationBar.isHidden = true
        viewController.delegate = self
        navigationController.pushViewController(viewController, animated: true)
    }
}

extension RedeemVoucherCoordinator: RedeemVoucherViewControllerDelegate {
    func redeemVoucherDidSucceed(
        _ controller: RedeemVoucherViewController,
        with response: REST.SubmitVoucherResponse
    ) {
        let coordinator = RedeemVoucherSucceededCoordinator(
            navigationController: navigationController,
            timeAdded: response.dateComponents
        )

        coordinator.didFinish = { [weak self] coordinator in
            guard let self else {
                return
            }
            dismiss(animated: true)
            didFinish?(self)
        }
        addChild(coordinator)
        coordinator.start()
    }

    func redeemVoucherDidCancel(_ controller: RedeemVoucherViewController) {
        didCancel?(self)
    }
}
