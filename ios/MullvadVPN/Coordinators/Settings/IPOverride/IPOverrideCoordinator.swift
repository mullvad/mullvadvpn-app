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
    private var interactor: IPOverrideInteractor
    private let repository: IPOverrideRepositoryProtocol
    private let tunnelManager: TunnelManager

    private lazy var ipOverrideViewController: IPOverrideViewController = {
        let viewController = IPOverrideViewController(
            interactor: interactor,
            alertPresenter: AlertPresenter(context: self)
        )
        viewController.delegate = self
        return viewController
    }()

    var presentationContext: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        repository: IPOverrideRepositoryProtocol,
        tunnelManager: TunnelManager
    ) {
        self.navigationController = navigationController
        self.repository = repository
        self.tunnelManager = tunnelManager

        interactor = IPOverrideInteractor(repository: repository)
    }

    func start(animated: Bool) {
        interactor.didUpdateOverrides = { [weak self] in
            self?.tunnelManager.reconnectTunnel(selectNewRelay: true)
        }

        navigationController.pushViewController(ipOverrideViewController, animated: animated)
    }
}

extension IPOverrideCoordinator: IPOverrideViewControllerDelegate {
    func presentImportTextController() {
        let viewController = IPOverrideTextViewController(interactor: interactor)
        let customNavigationController = CustomNavigationController(rootViewController: viewController)

        presentationContext.present(customNavigationController, animated: true)
    }
}
