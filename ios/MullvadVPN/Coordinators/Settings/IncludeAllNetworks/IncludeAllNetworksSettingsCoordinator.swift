//
//  IncludeAllNetworksSettingsCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-16.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Logging
import Routing
import SwiftUI

class IncludeAllNetworksSettingsCoordinator: Coordinator, SettingsChildCoordinator, Presentable, Presenting {
    private lazy var logger = Logger(label: "NotificationManager")
    private let navigationController: UINavigationController
    private let viewModel: IncludeAllNetworksSettingsViewModelImpl
    private let route: AppRoute

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((IncludeAllNetworksSettingsCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        route: AppRoute,
        viewModel: IncludeAllNetworksSettingsViewModelImpl
    ) {
        self.navigationController = navigationController
        self.route = route
        self.viewModel = viewModel

        super.init()
    }

    func start(animated: Bool) {
        let view = IncludeAllNetworksSettingsView(viewModel: self.viewModel)

        let host = UIHostingController(rootView: view)
        host.title = NSLocalizedString("Force all apps", comment: "")
        host.view.setAccessibilityIdentifier(.includeAllNetworksView)
        customiseNavigation(on: host)

        navigationController.pushViewController(host, animated: animated)
    }

    private func customiseNavigation(on viewController: UIViewController) {
        if route == .includeAllNetworks {
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
