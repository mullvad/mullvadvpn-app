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
        case .vpnSettings:
            makeVPNSettingsViewController()
        case .problemReport:
            makeProblemReportViewController()
        case .apiAccess:
            makeAPIAccessViewController()
        case .faq:
            // Handled separately and presented as a modal.
            .failed
        case .multihop:
            makeMultihopViewController()
        case .daita:
            makeDAITAViewController()
        }
    }

    private func makeVPNSettingsViewController() -> MakeChildResult {
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

    private func makeAPIAccessViewController() -> MakeChildResult {
        return .childCoordinator(ListAccessMethodCoordinator(
            navigationController: navigationController,
            accessMethodRepository: accessMethodRepository,
            proxyConfigurationTester: proxyConfigurationTester
        ))
    }

    private func makeMultihopViewController() -> MakeChildResult {
        let viewModel = MultihopTunnelSettingsViewModel(tunnelManager: interactorFactory.tunnelManager)
        let view = SettingsMultihopView(tunnelViewModel: viewModel)

        let host = UIHostingController(rootView: view)
        host.title = NSLocalizedString(
            "NAVIGATION_TITLE_MULTIHOP",
            tableName: "Settings",
            value: "Multihop",
            comment: ""
        )
        host.view.setAccessibilityIdentifier(.multihopView)

        return .viewController(host)
    }

    private func makeDAITAViewController() -> MakeChildResult {
        let viewModel = DAITATunnelSettingsViewModel(tunnelManager: interactorFactory.tunnelManager)
        let view = SettingsDAITAView(tunnelViewModel: viewModel)

        viewModel.didFailDAITAValidation = { result in
            showPrompt(
                for: result.item,
                onSave: {
                    viewModel.value = result.setting
                },
                onDiscard: {}
            )
        }

        let host = UIHostingController(rootView: view)
        host.title = NSLocalizedString(
            "NAVIGATION_TITLE_DAITA",
            tableName: "Settings",
            value: "DAITA",
            comment: ""
        )
        host.view.setAccessibilityIdentifier(.daitaView)

        return .viewController(host)
    }

    private func showPrompt(
        for item: DAITASettingsPromptItem,
        onSave: @escaping () -> Void,
        onDiscard: @escaping () -> Void
    ) {
        let presentation = AlertPresentation(
            id: "settings-daita-prompt",
            accessibilityIdentifier: .daitaPromptAlert,
            icon: .warning,
            message: NSLocalizedString(
                "SETTINGS_DAITA_ENABLE_TEXT",
                tableName: "DAITA",
                value: item.description,
                comment: ""
            ),
            buttons: [
                AlertAction(
                    title: String(format: NSLocalizedString(
                        "SETTINGS_DAITA_ENABLE_OK_ACTION",
                        tableName: "DAITA",
                        value: "Enable \"%@\"",
                        comment: ""
                    ), item.title),
                    style: .default,
                    accessibilityId: .daitaConfirmAlertEnableButton,
                    handler: { onSave() }
                ),
                AlertAction(
                    title: NSLocalizedString(
                        "SETTINGS_DAITA_ENABLE_CANCEL_ACTION",
                        tableName: "DAITA",
                        value: "Cancel",
                        comment: ""
                    ),
                    style: .default,
                    handler: { onDiscard() }
                ),
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }
}
