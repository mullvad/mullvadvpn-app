//
//  AddCreditSucceededCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-01.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

final class AddCreditSucceededCoordinator: Coordinator {
    var didFinish: ((AddCreditSucceededCoordinator) -> Void)?

    let timeAdded: Int
    let navigationController: RootContainerViewController

    init(timeAdded: Int, navigationController: RootContainerViewController) {
        self.timeAdded = timeAdded
        self.navigationController = navigationController
    }

    func start() {
        let controller =
            AddCreditSucceededViewController(timeAddedComponents: DateComponents(second: timeAdded))
        controller.delegate = self
        self.navigationController.pushViewController(controller, animated: true)
    }
}

extension AddCreditSucceededCoordinator: AddCreditSucceededViewControllerDelegate {
    func addCreditSucceededViewControllerDidFinish(_ controller: AddCreditSucceededViewController) {
        let coordinator = SetupAccountCompletedCoordinator(navigationController: navigationController)
        coordinator.didFinish = { [self] coordinator in
            coordinator.removeFromParent()
            didFinish?(self)
        }
        addChild(coordinator)
        coordinator.start(animated: true)
    }

    func titleForAction(in controller: AddCreditSucceededViewController) -> String {
        NSLocalizedString(
            "REDEEM_VOUCHER_DISMISS_BUTTON",
            tableName: "Welcome",
            value: "Next",
            comment: ""
        )
    }
}
