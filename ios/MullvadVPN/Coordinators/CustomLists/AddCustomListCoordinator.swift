//
//  AddCustomListCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import Routing
import UIKit

class AddCustomListCoordinator: Coordinator, Presentable, Presenting {
    let navigationController: UINavigationController
    let interactor: any CustomListInteractorProtocol
    let nodes: [LocationNode]
    let subject = CurrentValueSubject<CustomListViewModel, Never>(
        CustomListViewModel(id: UUID(), name: "", locations: [], tableSections: [.name, .addLocations])
    )

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((AddCustomListCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        interactor: any CustomListInteractorProtocol,
        nodes: [LocationNode]
    ) {
        self.navigationController = navigationController
        self.interactor = interactor
        self.nodes = nodes
    }

    func start() {
        let controller = CustomListViewController(
            interactor: interactor,
            subject: subject,
            alertPresenter: AlertPresenter(context: self)
        )
        controller.delegate = self

        controller.navigationItem.title = NSLocalizedString(
            "CUSTOM_LISTS_NAVIGATION_EDIT_TITLE",
            tableName: "CustomLists",
            value: "New custom list",
            comment: ""
        )

        controller.saveBarButton.title = NSLocalizedString(
            "CUSTOM_LISTS_NAVIGATION_CREATE_BUTTON",
            tableName: "CustomLists",
            value: "Create",
            comment: ""
        )

        controller.navigationItem.leftBarButtonItem = UIBarButtonItem(
            systemItem: .cancel,
            primaryAction: UIAction(handler: { [weak self] _ in
                guard let self else {
                    return
                }
                didFinish?(self)
            })
        )

        navigationController.pushViewController(controller, animated: false)
    }
}

extension AddCustomListCoordinator: CustomListViewControllerDelegate {
    func customListDidSave(_ list: CustomList) {
        didFinish?(self)
    }

    func customListDidDelete(_ list: CustomList) {
        // No op.
    }

    func showLocations(_ list: CustomList) {
        let coordinator = AddLocationsCoordinator(
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
