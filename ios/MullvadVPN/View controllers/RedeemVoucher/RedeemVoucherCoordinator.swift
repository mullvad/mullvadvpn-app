//
//  RedeemVoucherCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class RedeemVoucherCoordinator: Coordinator, Presentable {
    let navigationController: UINavigationController
    let interactor: RedeemVoucherInteractor
    init(
        navigationController: UINavigationController,
        interactor: RedeemVoucherInteractor
    ) {
        self.navigationController = navigationController
        self.interactor = interactor
    }

    var presentedViewController: UIViewController {
        return navigationController
    }

    func start() {
        navigationController.navigationBar.prefersLargeTitles = true

        let controller = AccountRedeemVoucherController(interactor: interactor)

        navigationController.present(controller, animated: true)
    }
}
