//
//  EditCustomListCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import Routing
import UIKit

class EditCustomListCoordinator: Coordinator, Presentable, Presenting {
    enum FinishAction {
        case save, delete
    }

    let navigationController: UINavigationController
    let customListInteractor: CustomListInteractorProtocol
    let customList: CustomList
    let nodes: [LocationNode]
    let subject: CurrentValueSubject<CustomListViewModel, Never>

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((EditCustomListCoordinator, FinishAction, CustomList) -> Void)?

    init(
        navigationController: UINavigationController,
        customListInteractor: CustomListInteractorProtocol,
        customList: CustomList,
        nodes: [LocationNode]
    ) {
        self.navigationController = navigationController
        self.customListInteractor = customListInteractor
        self.customList = customList
        self.nodes = nodes
        self.subject = CurrentValueSubject(CustomListViewModel(
            id: customList.id,
            name: customList.name,
            locations: customList.locations,
            tableSections: [.name, .editLocations, .deleteList]
        ))
    }

    func start() {
        let controller = CustomListViewController(
            interactor: customListInteractor,
            subject: subject,
            alertPresenter: AlertPresenter(context: self)
        )
        controller.delegate = self

        controller.navigationItem.title = NSLocalizedString(
            "CUSTOM_LIST_NAVIGATION_TITLE",
            tableName: "CustomLists",
            value: subject.value.name,
            comment: ""
        )

        navigationController.pushViewController(controller, animated: true)
    }
}

extension EditCustomListCoordinator: CustomListViewControllerDelegate {
    func customListDidSave(_ list: CustomList) {
        didFinish?(self, .save, list)
    }

    func customListDidDelete(_ list: CustomList) {
        didFinish?(self, .delete, list)
    }

    func showLocations(_ list: CustomList) {
        let coordinator = EditLocationsCoordinator(
            navigationController: navigationController,
            nodes: nodes,
            customList: list
        )

        coordinator.didFinish = { [weak self] locationsCoordinator, customList in
            guard let self else { return }
            subject.send(CustomListViewModel(
                id: customList.id,
                name: customList.name,
                locations: customList.locations,
                tableSections: subject.value.tableSections
            ))
            locationsCoordinator.removeFromParent()
        }

        coordinator.start()

        addChild(coordinator)
    }
}
