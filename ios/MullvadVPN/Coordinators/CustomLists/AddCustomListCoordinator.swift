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
    let customListInteractor: CustomListInteractorProtocol

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: (() -> Void)?

    init(
        navigationController: UINavigationController,
        customListInteractor: CustomListInteractorProtocol
    ) {
        self.navigationController = navigationController
        self.customListInteractor = customListInteractor
    }

    func start() {
        let subject = CurrentValueSubject<CustomListViewModel, Never>(
            CustomListViewModel(id: UUID(), name: "", locations: [], tableSections: [.name, .addLocations])
        )

        let controller = CustomListViewController(
            interactor: customListInteractor,
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

        navigationController.pushViewController(controller, animated: false)
    }
}

extension AddCustomListCoordinator: CustomListViewControllerDelegate {
    func customListDidSave() {
        didFinish?()
    }

    func customListDidDelete() {
        // No op.
    }

    func showLocations() {
        // TODO: Show view controller for locations.
    }
}
