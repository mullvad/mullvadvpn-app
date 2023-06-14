//
//  RedeemVoucherSucceededCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-14.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

final class RedeemVoucherSucceededCoordinator: Coordinator, Presentable {
    private let navigationController: UINavigationController
    private let viewController: RedeemVoucherSucceededViewController

    var didFinish: ((RedeemVoucherSucceededCoordinator) -> Void)?

    var presentedViewController: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        timeAdded: DateComponents
    ) {
        self.navigationController = navigationController
        viewController = RedeemVoucherSucceededViewController(timeAddedComponents: timeAdded)
    }

    func start() {
        navigationController.navigationBar.isHidden = true
        viewController.delegate = self
        navigationController.pushViewController(viewController, animated: false)
    }
}

extension RedeemVoucherSucceededCoordinator: RedeemVoucherSucceededViewControllerDelegate {
    func redeemVoucherSucceededViewControllerDidFinish(_ controller: RedeemVoucherSucceededViewController) {
        didFinish?(self)
    }
}
