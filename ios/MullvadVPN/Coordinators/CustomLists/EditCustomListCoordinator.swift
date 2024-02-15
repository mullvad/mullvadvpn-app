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

    func start(customList: CustomList) {
        let subject = CurrentValueSubject<CustomListViewModel, Never>(
            CustomListViewModel(
                id: customList.id,
                name: customList.name,
                locations: customList.locations,
                tableSections: [.name, .editLocations, .deleteList]
            )
        )

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

        navigationController.pushViewController(controller, animated: false)
    }
}

extension EditCustomListCoordinator: CustomListViewControllerDelegate {
    func customListDidSave() {
        didFinish?()
    }

    func customListDidDelete() {
        didFinish?()
    }

    func showLocations() {
        // TODO: Show view controller for locations.
    }
}
