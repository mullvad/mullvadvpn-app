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
    let subject: CurrentValueSubject<CustomListViewModel, Never>

    lazy var alertPresenter: AlertPresenter = {
        AlertPresenter(context: self)
    }()

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

        subject = CurrentValueSubject<CustomListViewModel, Never>(
            CustomListViewModel(
                id: UUID(),
                name: "A list",
                locations: [],
                tableSections: [.name, .editLocations, .deleteList]
            )
        )
    }

    func start() {
        let controller = CustomListViewController(interactor: customListInteractor, subject: subject)
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
        let presentation = AlertPresentation(
            id: "api-custom-lists-delete-list-alert",
            icon: .alert,
            message: NSLocalizedString(
                "CUSTOM_LISTS_DELETE_PROMPT",
                tableName: "APIAccess",
                value: "Delete \(subject.value.name)?",
                comment: ""
            ),
            buttons: [
                AlertAction(
                    title: NSLocalizedString(
                        "CUSTOM_LISTS_DELETE_BUTTON",
                        tableName: "APIAccess",
                        value: "Delete",
                        comment: ""
                    ),
                    style: .destructive,
                    handler: {
                        self.customListInteractor.deleteCustomList(id: self.subject.value.id)
                        self.dismiss(animated: true)
                        self.didFinish?()
                    }
                ),
                AlertAction(
                    title: NSLocalizedString(
                        "CUSTOM_LISTS_CANCEL_BUTTON",
                        tableName: "APIAccess",
                        value: "Cancel",
                        comment: ""
                    ),
                    style: .default
                )
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }

    func showLocations() {
        // TODO: Show view controller for locations.
    }
}
