//
//  DAITASettingsCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-20.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Routing
import SwiftUI

class DAITASettingsCoordinator: Coordinator, SettingsChildCoordinator, Presentable, Presenting {
    private let navigationController: UINavigationController
    private let viewModel: DAITATunnelSettingsViewModel
    private var alertPresenter: AlertPresenter?
    private let route: AppRoute

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((DAITASettingsCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        route: AppRoute,
        viewModel: DAITATunnelSettingsViewModel
    ) {
        self.navigationController = navigationController
        self.route = route
        self.viewModel = viewModel

        super.init()

        alertPresenter = AlertPresenter(context: self)
    }

    func start(animated: Bool) {
        let view = SettingsDAITAView(tunnelViewModel: self.viewModel)

        viewModel.didFailDAITAValidation = { [weak self] result in
            guard let self else { return }

            showPrompt(
                for: result.item,
                onSave: {
                    self.viewModel.value = result.setting
                },
                onDiscard: {}
            )
        }

        let host = UIHostingController(rootView: view)
        host.title = NSLocalizedString("DAITA", comment: "")
        host.view.setAccessibilityIdentifier(.daitaView)
        customiseNavigation(on: host)

        navigationController.pushViewController(host, animated: animated)
    }

    private func customiseNavigation(on viewController: UIViewController) {
        if route == .daita {
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

    private func showPrompt(
        for item: DAITASettingsPromptItem,
        onSave: @escaping () -> Void,
        onDiscard: @escaping () -> Void
    ) {
        let presentation = AlertPresentation(
            id: "settings-daita-prompt",
            accessibilityIdentifier: .daitaPromptAlert,
            icon: .warning,
            message: item.description,
            buttons: [
                AlertAction(
                    title: "\(NSLocalizedString("Enable", comment: "")) \"\(item.title)\"",
                    style: .default,
                    accessibilityId: .daitaConfirmAlertEnableButton,
                    handler: { onSave() }
                ),
                AlertAction(
                    title: NSLocalizedString("Cancel", comment: ""),
                    style: .default,
                    handler: { onDiscard() }
                ),
            ]
        )

        alertPresenter?.showAlert(presentation: presentation, animated: true)
    }
}
