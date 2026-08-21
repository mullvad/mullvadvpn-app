// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadLogging
import Routing
import SwiftUI
import UIKit

final class ChangeLogCoordinator: Coordinator, Presentable, SettingsChildCoordinator {
    private let route: AppRoute
    private let viewModel: ChangeLogViewModel
    private var navigationController: UINavigationController?
    var didFinish: ((ChangeLogCoordinator) -> Void)?

    var presentedViewController: UIViewController {
        navigationController!
    }

    init(
        route: AppRoute,
        navigationController: UINavigationController,
        viewModel: ChangeLogViewModel
    ) {
        self.route = route
        self.viewModel = viewModel
        self.navigationController = navigationController
    }

    func start(animated: Bool) {
        let changeLogViewController = UIHostingController(rootView: ChangeLogView(viewModel: viewModel))
        changeLogViewController.view.setAccessibilityIdentifier(.changeLogAlert)
        changeLogViewController.navigationItem.title = NSLocalizedString("What’s new", comment: "")

        switch route {
        case .changelog:
            let barButtonItem = UIBarButtonItem(
                title: NSLocalizedString("Done", comment: ""),
                primaryAction: UIAction { [weak self] _ in
                    guard let self else { return }
                    didFinish?(self)
                }
            )
            barButtonItem.style = .done
            changeLogViewController.navigationItem.rightBarButtonItem = barButtonItem
            fallthrough
        case .settings:
            changeLogViewController.navigationItem.largeTitleDisplayMode = .always
            navigationController?.navigationBar.prefersLargeTitles = true
        default: break
        }

        navigationController?.pushViewController(changeLogViewController, animated: animated)
    }
}
