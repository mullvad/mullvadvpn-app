//
//  IPOverrideCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class IPOverrideCoordinator: Coordinator, Presenting, SettingsChildCoordinator {
    private let navigationController: UINavigationController
    private let interactor: IPOverrideInteractor

    var presentationContext: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        repository: IPOverrideRepositoryProtocol,
        tunnelManager: TunnelManager
    ) {
        self.navigationController = navigationController
        interactor = IPOverrideInteractor(repository: repository, tunnelManager: tunnelManager)
    }

    func start(animated: Bool) {
        let controller = IPOverrideViewController(
            interactor: interactor,
            alertPresenter: AlertPresenter(context: self)
        )

        controller.delegate = self

        navigationController.pushViewController(controller, animated: animated)
    }
}

extension IPOverrideCoordinator: IPOverrideViewControllerDelegate {
    func presentImportTextController() {
        let viewController = IPOverrideTextViewController(interactor: interactor)
        let customNavigationController = CustomNavigationController(rootViewController: viewController)

        presentationContext.present(customNavigationController, animated: true)
    }
}
