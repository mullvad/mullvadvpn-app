//
//  IPOverrideCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import Routing
import UIKit

class IPOverrideCoordinator: Coordinator, Presenting, SettingsChildCoordinator {
    let navigationController: UINavigationController

    var presentationContext: UIViewController {
        navigationController
    }

    init(navigationController: UINavigationController) {
        self.navigationController = navigationController
    }

    func start(animated: Bool) {
        let viewController = IPOverrideViewController()
        navigationController.pushViewController(viewController, animated: animated)
    }
}
