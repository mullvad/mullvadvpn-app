// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class ListCustomListCoordinator: Coordinator, Presentable, Presenting {
    let navigationController: UINavigationController
    let interactor: CustomListInteractorProtocol
    let tunnelManager: TunnelManager
    let listViewController: ListCustomListViewController
    let nodes: [LocationNode]

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((ListCustomListCoordinator, CustomListAction) -> Void)?

    init(
        navigationController: UINavigationController,
        interactor: CustomListInteractorProtocol,
        tunnelManager: TunnelManager,
        nodes: [LocationNode]
    ) {
        self.navigationController = navigationController
        self.interactor = interactor
        self.tunnelManager = tunnelManager
        self.nodes = nodes

        listViewController = ListCustomListViewController(interactor: interactor)
    }

    func start() {
        listViewController.didFinish = { [weak self] action in
            guard let self else { return }
            didFinish?(self, action)
        }
        listViewController.didSelectItem = { [weak self] in
            self?.edit(list: $0)
        }

        navigationController.pushViewController(listViewController, animated: false)
    }

    private func edit(list: CustomList) {
        let coordinator = EditCustomListCoordinator(
            navigationController: navigationController,
            customListInteractor: interactor,
            customList: list,
            nodes: nodes
        )

        coordinator.didFinish = { [weak self] editCustomListCoordinator, action in
            guard let self else { return }

            popToList(action)
            editCustomListCoordinator.removeFromParent()
        }

        coordinator.didCancel = { [weak self] editCustomListCoordinator in
            guard let self else { return }
            popToList(.noAction)
            editCustomListCoordinator.removeFromParent()
        }

        coordinator.start()
        addChild(coordinator)
    }

    private func popToList(_ action: CustomListAction) {
        if interactor.fetchAll().isEmpty {
            navigationController.dismiss(animated: true)
            didFinish?(self, action)
        } else if let listController = navigationController.viewControllers
            .first(where: { $0 is ListCustomListViewController })
        {
            navigationController.popToViewController(listController, animated: true)
            listViewController.updateDataSource(reloadExisting: true, animated: false)
        }
    }
}
