//
//  VPNSettingsCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-03-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class VPNSettingsCoordinator: Coordinator, Presenting, SettingsChildCoordinator {
    private let navigationController: UINavigationController
    private let interactorFactory: SettingsInteractorFactory
    private let ipOverrideRepository: IPOverrideRepositoryProtocol

    var presentationContext: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        interactorFactory: SettingsInteractorFactory,
        ipOverrideRepository: IPOverrideRepositoryProtocol
    ) {
        self.navigationController = navigationController
        self.interactorFactory = interactorFactory
        self.ipOverrideRepository = ipOverrideRepository
    }

    func start(animated: Bool) {
        let controller = VPNSettingsViewController(
            interactor: interactorFactory.makeVPNSettingsInteractor(),
            alertPresenter: AlertPresenter(context: self)
        )

        controller.delegate = self

        navigationController.pushViewController(controller, animated: animated)
    }
}

extension VPNSettingsCoordinator: @preconcurrency VPNSettingsViewControllerDelegate {
    func showIPOverrides() {
        let coordinator = IPOverrideCoordinator(
            navigationController: navigationController,
            repository: ipOverrideRepository,
            tunnelManager: interactorFactory.tunnelManager
        )

        addChild(coordinator)
        coordinator.start(animated: true)
    }
}
