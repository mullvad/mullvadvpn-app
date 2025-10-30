//
//  ListCustomListCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-03-06.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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

    var didFinish: ((ListCustomListCoordinator) -> Void)?

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
        listViewController.didFinish = { [weak self] in
            guard let self else { return }
            didFinish?(self)
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

        coordinator.didFinish = { [weak self] editCustomListCoordinator, list in
            guard let self else { return }

            popToList()
            editCustomListCoordinator.removeFromParent()
        }

        coordinator.didCancel = { [weak self] editCustomListCoordinator in
            guard let self else { return }
            popToList()
            editCustomListCoordinator.removeFromParent()
        }

        coordinator.start()
        addChild(coordinator)
    }

    private func popToList() {
        if interactor.fetchAll().isEmpty {
            navigationController.dismiss(animated: true)
            didFinish?(self)
        } else if let listController = navigationController.viewControllers
            .first(where: { $0 is ListCustomListViewController })
        {
            navigationController.popToViewController(listController, animated: true)
            listViewController.updateDataSource(reloadExisting: true, animated: false)
        }
    }
}
