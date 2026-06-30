//
//  SettingsMigrationWizardCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-05-08.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Logging
import MullvadSettings
import Routing
import SwiftUI

class SettingsMigrationWizardCoordinator: Coordinator, SettingsChildCoordinator, Presentable, Presenting {
    private let logger = Logger(label: "SettingsMigrationWizardCoordinator")
    private let navigationController: UINavigationController
    private let viewModel: SettingsMigrationWizardViewModel
    private let route: AppRoute

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((SettingsMigrationWizardCoordinator, Bool) -> Void)?

    init(
        navigationController: UINavigationController,
        route: AppRoute,
        viewModel: SettingsMigrationWizardViewModel
    ) {
        self.navigationController = navigationController
        self.route = route
        self.viewModel = viewModel
        super.init()
    }

    func start(animated: Bool) {
        let view = SettingsMigrationWizardView(viewModel: viewModel) {
            [weak self] in
            guard let self else { return }
            logger.info("the migrated settings wizard has completed")
            didFinish?(self, true)
        }

        let host = UIHostingController(rootView: view)
        host.view.setAccessibilityIdentifier(.settingsMigrationCompleteView)
        host.navigationItem.largeTitleDisplayMode = .never
        customiseNavigation(on: host)

        navigationController.pushViewController(host, animated: animated)
    }

    private func customiseNavigation(on viewController: UIViewController) {
        if route == .settingsMigrationWizard {
            let closeButton = UIBarButtonItem(
                image: .Buttons.close,
                primaryAction: UIAction(handler: { [weak self] _ in
                    guard let self else { return }
                    didFinish?(self, false)
                })
            )
            viewController.navigationItem.leftBarButtonItem = closeButton
        }
    }
}
