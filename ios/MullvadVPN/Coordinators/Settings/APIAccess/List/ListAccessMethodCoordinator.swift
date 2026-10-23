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
            proxyConfigurationTester: proxyConfigurationTester)
        self.route = route
    }

    func start(animated: Bool) {
        let view = ListAccessMethodView(factory: listAccessMethodViewFactory)

        let host = UIHostingController(rootView: view)
        //host.view.setAccessibilityIdentifier(.apiAccessView)
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

    //    private func addNew() {
    //        let coordinator = AddAccessMethodCoordinator(
    //            navigationController: CustomNavigationController(),
    //            accessMethodRepo: accessMethodRepository,
    //            proxyConfigurationTester: proxyConfigurationTester
    //        )
    //
    //        coordinator.start()
    //        presentChild(coordinator, animated: true)
    //    }

    private func edit(item: ListAccessMethodItem) {
        //        // Remove previous edit coordinator to prevent accumulation.
        //        childCoordinators.filter { $0 is EditAccessMethodCoordinator }.forEach { $0.removeFromParent() }
        //
        //        let editCoordinator = EditAccessMethodCoordinator(
        //            navigationController: navigationController,
        //            accessMethodRepo: accessMethodRepository,
        //            proxyConfigurationTester: proxyConfigurationTester,
        //            methodIdentifier: item.id
        //        )
        //        editCoordinator.onFinish = { [weak self] coordinator in
        //            guard let self else { return }
        //
        //            popToList()
        //            coordinator.removeFromParent()
        //
        //            let methods = accessMethodRepository.fetchAll()
        //            let ciphers = accessMethodRepository.shadowsocksCiphers
        //
        //            let methodsWithInvalidCiphers = methods.filter { method in
        //                if case .shadowsocks(let config) = method.proxyConfiguration {
        //                    !ciphers.contains(config.cipher)
        //                } else {
        //                    false
        //                }
        //            }
        //
        //            if methodsWithInvalidCiphers.isEmpty {
        //                breadcrumbsProvider.remove(breadcrumb: .warning(.apiAccess))
        //            }
        //        }
        //        editCoordinator.start()
        //        addChild(editCoordinator)
    }
}
