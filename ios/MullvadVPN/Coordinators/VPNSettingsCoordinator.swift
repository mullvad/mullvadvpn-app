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
}

class VPNSettingsCoordinator: Coordinator, Presenting, Presentable, SettingsChildCoordinator {
    private let navigationController: UINavigationController
    private let interactorFactory: SettingsInteractorFactory
    private let ipOverrideRepository: IPOverrideRepositoryProtocol
    private let section: VPNSettingsSection?

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
        section: VPNSettingsSection?
    ) {
        self.navigationController = navigationController
        self.interactorFactory = interactorFactory
        self.ipOverrideRepository = ipOverrideRepository
        self.section = section
    }

    func start(animated: Bool) {
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
        if section != nil {
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
            tunnelManager: interactorFactory.tunnelManager
        )

        addChild(coordinator)
        coordinator.start(animated: true)
    }
}
