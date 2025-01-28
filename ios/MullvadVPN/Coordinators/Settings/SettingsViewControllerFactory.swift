//
//  SettingsViewControllerFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-26.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import Routing
import SwiftUI
import UIKit

@MainActor
struct SettingsViewControllerFactory {
    /// The result of creating a child representing a route.
    enum MakeChildResult {
        /// View controller that should be pushed into navigation stack.
        case viewController(UIViewController)

        /// Child coordinator that should be added to the children hierarchy.
        /// The child is responsile for presenting itself.
        case childCoordinator(SettingsChildCoordinator)

        /// Failure to produce a child.
        case failed
    }

    private let interactorFactory: SettingsInteractorFactory
    private let accessMethodRepository: AccessMethodRepositoryProtocol
    private let proxyConfigurationTester: ProxyConfigurationTesterProtocol
    private let ipOverrideRepository: IPOverrideRepository
    private let navigationController: UINavigationController
    private let alertPresenter: AlertPresenter

    init(
        interactorFactory: SettingsInteractorFactory,
        accessMethodRepository: AccessMethodRepositoryProtocol,
        proxyConfigurationTester: ProxyConfigurationTesterProtocol,
        ipOverrideRepository: IPOverrideRepository,
        navigationController: UINavigationController,
        alertPresenter: AlertPresenter
    ) {
        self.interactorFactory = interactorFactory
        self.accessMethodRepository = accessMethodRepository
        self.proxyConfigurationTester = proxyConfigurationTester
        self.ipOverrideRepository = ipOverrideRepository
        self.navigationController = navigationController
        self.alertPresenter = alertPresenter
    }

    func makeRoute(for route: SettingsNavigationRoute) -> MakeChildResult {
        switch route {
        case .root:
            // Handled in SettingsCoordinator.
            .failed
        case .faq:
            // Handled separately and presented as a modal.
            .failed
        case .vpnSettings:
            makeVPNSettingsViewCoordinator()
        case .problemReport:
            makeProblemReportViewController()
        case .apiAccess:
            makeAPIAccessCoordinator()
        case .changelog:
            makeChangelogCoordinator()
        case .multihop:
            makeMultihopViewController()
        case .daita:
            makeDAITASettingsCoordinator()
        }
    }

    private func makeVPNSettingsViewCoordinator() -> MakeChildResult {
        return .childCoordinator(VPNSettingsCoordinator(
            navigationController: navigationController,
            interactorFactory: interactorFactory,
            ipOverrideRepository: ipOverrideRepository
        ))
    }

    private func makeProblemReportViewController() -> MakeChildResult {
        return .viewController(ProblemReportViewController(
            interactor: interactorFactory.makeProblemReportInteractor(),
            alertPresenter: alertPresenter
        ))
    }

    private func makeAPIAccessCoordinator() -> MakeChildResult {
        return .childCoordinator(ListAccessMethodCoordinator(
            navigationController: navigationController,
            accessMethodRepository: accessMethodRepository,
            proxyConfigurationTester: proxyConfigurationTester
        ))
    }

    private func makeChangelogCoordinator() -> MakeChildResult {
        return .childCoordinator(
            ChangeLogCoordinator(
                route: .settings(.changelog),
                navigationController: navigationController,
                viewModel: ChangeLogViewModel(changeLogReader: ChangeLogReader())
            )
        )
    }

    private func makeMultihopViewController() -> MakeChildResult {
        let viewModel = MultihopTunnelSettingsViewModel(tunnelManager: interactorFactory.tunnelManager)
        let view = SettingsMultihopView(tunnelViewModel: viewModel)

        let viewController = MultihopViewController(rootView: view)
        viewController.title = NSLocalizedString(
            "NAVIGATION_TITLE_MULTIHOP",
            tableName: "Settings",
            value: "Multihop",
            comment: ""
        )

        return .viewController(viewController)
    }

    private func makeDAITASettingsCoordinator() -> MakeChildResult {
        let viewModel = DAITATunnelSettingsViewModel(tunnelManager: interactorFactory.tunnelManager)
        let coordinator = DAITASettingsCoordinator(
            navigationController: navigationController,
            route: .settings(.daita),
            viewModel: viewModel
        )

        return .childCoordinator(coordinator)
    }
}
