//
//  VPNSettingsCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-03-18.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import Routing
import SwiftUI
import UIKit

enum VPNSettingsSection: Equatable {
    case quantumResistance
    case obfuscation
    case ipVersion
    case none
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
        let section: VPNSettingsSection =
            if case let .vpnSettings(route) = route { route } else {
                .none
            }
        let alertPresenter = AlertPresenter(context: self)

        let view = VPNSettingsNavigationView(
            settingsInteractor: interactorFactory.makeVPNSettingsInteractor(),
            IPOverrideInteractor: interactorFactory.makeIPOverrideInteractor(),
            alertPresenter: alertPresenter,
            navigationController: navigationController,
            presentOnlySection: section
        )

        let host = UIHostingController(rootView: view)
        customiseNavigation(on: host)

        navigationController.pushViewController(host, animated: animated)
    }

    private func customiseNavigation(on viewController: UIViewController) {
        if case .vpnSettings = route {
            navigationController.navigationItem.largeTitleDisplayMode = .always
            navigationController.navigationBar.prefersLargeTitles = true

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
