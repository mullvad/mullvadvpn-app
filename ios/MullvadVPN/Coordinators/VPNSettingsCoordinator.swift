//
//  VPNSettingsCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-03-18.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import Routing
import UIKit

enum VPNSettingsSection: Equatable {
    case quantumResistance
    case obfuscation
    case ipVersion
}

class VPNSettingsCoordinator: Coordinator, Presenting, Presentable, SettingsChildCoordinator {
    private let navigationController: UINavigationController
    private let interactorFactory: SettingsInteractorFactory
    private let ipOverrideRepository: IPOverrideRepositoryProtocol
    private let route: AppRoute

    var presentationContext: UIViewController {
        navigationController
    }

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((VPNSettingsCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        interactorFactory: SettingsInteractorFactory,
        ipOverrideRepository: IPOverrideRepositoryProtocol,
        route: AppRoute
    ) {
        self.navigationController = navigationController
        self.interactorFactory = interactorFactory
        self.ipOverrideRepository = ipOverrideRepository
        self.route = route
    }

    func start(animated: Bool) {
        let section: VPNSettingsSection? =
            if case let .vpnSettings(route) = route { route } else {
                nil
            }
        let controller = VPNSettingsViewController(
            interactor: interactorFactory.makeVPNSettingsInteractor(),
            alertPresenter: AlertPresenter(context: self),
            section: section
        )

        controller.delegate = self
        customiseNavigation(on: controller)
        navigationController.pushViewController(controller, animated: animated)
    }

    private func customiseNavigation(on viewController: UIViewController) {
        if case .vpnSettings = route {
            let doneButton = UIBarButtonItem(
                systemItem: .done,
                primaryAction: UIAction(handler: { [weak self] _ in
                    guard let self else { return }
                    didFinish?(self)
                })
            )
            viewController.navigationItem.rightBarButtonItem = doneButton
        }
    }
}

extension VPNSettingsCoordinator: @preconcurrency VPNSettingsViewControllerDelegate {
    func showIPOverrides() {
        let coordinator = IPOverrideCoordinator(
            navigationController: navigationController,
            repository: ipOverrideRepository,
            tunnelManager: interactorFactory.tunnelManager,
            route: nil
        )

        addChild(coordinator)
        coordinator.start(animated: true)
    }
}
