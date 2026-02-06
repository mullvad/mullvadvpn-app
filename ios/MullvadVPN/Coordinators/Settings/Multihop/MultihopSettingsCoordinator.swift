//
//  DAITASettingsCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Routing
import SwiftUI

class MultihopSettingsCoordinator: Coordinator, SettingsChildCoordinator, Presentable, Presenting {
    private let navigationController: UINavigationController
    private let viewModel: MultihopTunnelSettingsViewModel
    private var alertPresenter: AlertPresenter?
    private let route: AppRoute

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((MultihopSettingsCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        route: AppRoute,
        viewModel: MultihopTunnelSettingsViewModel
    ) {
        self.navigationController = navigationController
        self.route = route
        self.viewModel = viewModel

        super.init()

        alertPresenter = AlertPresenter(context: self)
    }

    func start(animated: Bool) {
        let view = SettingsMultihopView(tunnelViewModel: self.viewModel)

        let host = UIHostingController(rootView: view)
        host.title = NSLocalizedString("Multihop", comment: "")
        host.view.setAccessibilityIdentifier(.multihopView)
        customiseNavigation(on: host)

        navigationController.pushViewController(host, animated: animated)
    }

    private func customiseNavigation(on viewController: UIViewController) {
        if route == .multihop {
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
