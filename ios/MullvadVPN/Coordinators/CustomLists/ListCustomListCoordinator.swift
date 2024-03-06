//
//  ListCustomListCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-03-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import Routing
import UIKit

class ListCustomListCoordinator: Coordinator, Presentable, Presenting {
    let navigationController: UINavigationController
    let interactor: CustomListInteractorProtocol
    let listViewController: ListCustomListViewController

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: (() -> Void)?

    init(
        navigationController: UINavigationController,
        interactor: CustomListInteractorProtocol
    ) {
        self.navigationController = navigationController
        self.interactor = interactor
        listViewController = ListCustomListViewController(interactor: interactor)
    }

    func start() {
        listViewController.didFinish = didFinish
        listViewController.didSelectItem = {
            self.edit(list: $0)
        }

        navigationController.pushViewController(listViewController, animated: false)
    }

    private func edit(list: CustomList) {
        // Remove previous edit coordinator to prevent accumulation.
        childCoordinators.filter { $0 is EditCustomListCoordinator }.forEach { $0.removeFromParent() }

        let coordinator = EditCustomListCoordinator(
            navigationController: navigationController,
            customListInteractor: interactor,
            customList: list
        )

        coordinator.didFinish = {
            self.popToList()
            coordinator.removeFromParent()

            self.listViewController.updateDataSource()
        }

        coordinator.start()
        addChild(coordinator)
    }

    private func popToList() {
        guard let listController = navigationController.viewControllers
            .first(where: { $0 is ListCustomListViewController }) else { return }

        navigationController.popToViewController(listController, animated: true)
    }
}
