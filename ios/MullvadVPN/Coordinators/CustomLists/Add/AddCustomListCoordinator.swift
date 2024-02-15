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

class AddCustomListCoordinator: Coordinator, Presentable {
    let navigationController: UINavigationController
    let customListInteractor: CustomListInteractorProtocol

    var presentedViewController: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        customListInteractor: CustomListInteractorProtocol
    ) {
        self.navigationController = navigationController
        self.customListInteractor = customListInteractor
    }

    func start() {
        let subject = CurrentValueSubject<CustomListViewModel, Never>(
            CustomListViewModel(id: UUID(), name: "", locations: [])
        )
        let controller = AddCustomListViewController(interactor: customListInteractor, subject: subject)

        controller.delegate = self

        navigationController.pushViewController(controller, animated: false)
    }
}

extension AddCustomListCoordinator: AddCustomListViewControllerDelegate {
    func customListDidSave() {
        dismiss(animated: true)
    }

    func showLocations() {
        // TODO: Show view controller for locations.
    }
}
