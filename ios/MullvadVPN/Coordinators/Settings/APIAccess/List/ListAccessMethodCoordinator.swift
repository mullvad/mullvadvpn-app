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
import MullvadSettings
import Routing
import SwiftUI
import UIKit

class ListAccessMethodCoordinator: Coordinator, Presenting, Presentable, SettingsChildCoordinator {
    let navigationController: UINavigationController
    let accessMethodRepository: AccessMethodRepositoryProtocol
    let proxyConfigurationTester: ProxyConfigurationTesterProtocol
    let breadcrumbsProvider: BreadcrumbsProvider
    let route: AppRoute
    let listAccessMethodViewFactory: ListAccessMethodViewFactory

    let logger = Logger(label: "ListAccessMethodCoordinator")

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((ListAccessMethodCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        accessMethodRepository: AccessMethodRepositoryProtocol,
        proxyConfigurationTester: ProxyConfigurationTesterProtocol,
        breadcrumbsProvider: BreadcrumbsProvider,
        route: AppRoute
    ) {
        self.navigationController = navigationController
        self.accessMethodRepository = accessMethodRepository
        self.proxyConfigurationTester = proxyConfigurationTester
        self.breadcrumbsProvider = breadcrumbsProvider
        self.listAccessMethodViewFactory = ListAccessMethodViewFactory(
            repository: accessMethodRepository,
            proxyConfigurationTester: proxyConfigurationTester,
            breadcrumbsProvider: breadcrumbsProvider)
        self.route = route
    }

    func start(animated: Bool) {
        let view = ListAccessMethodView(factory: listAccessMethodViewFactory)

        let host = UIHostingController(rootView: view)
        customiseNavigation(on: host)
        navigationController.pushViewController(host, animated: animated)
    }

    private func customiseNavigation(on viewController: UIViewController) {
        if route == .apiAccess {
            navigationController.navigationItem.largeTitleDisplayMode = .always
            navigationController.navigationBar.prefersLargeTitles = true

            let doneButton = UIBarButtonItem.mullvadDoneButton(
                primaryAction: UIAction(handler: { [weak self] _ in
                    guard let self else { return }
                    didFinish?(self)
                })
            )
            viewController.navigationItem.rightBarButtonItem = doneButton
        }
    }
}
