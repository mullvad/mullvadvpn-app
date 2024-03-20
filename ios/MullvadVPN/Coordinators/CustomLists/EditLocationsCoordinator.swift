//
//  EditLocationsCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-03-07.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class EditLocationsCoordinator: Coordinator, Presentable, Presenting {
    private let navigationController: UINavigationController
    private let nodes: [LocationNode]
    private var customList: CustomList

    var didFinish: ((EditLocationsCoordinator, CustomList) -> Void)?

    var presentedViewController: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        nodes: [LocationNode],
        customList: CustomList
    ) {
        self.navigationController = navigationController
        self.nodes = nodes
        self.customList = customList
    }

    func start() {
        let controller = AddLocationsViewController(
            allLocationsNodes: nodes,
            customList: customList
        )
        controller.delegate = self

        controller.navigationItem.title = NSLocalizedString(
            "EDIT_LOCATIONS_NAVIGATION_TITLE",
            tableName: "EditLocations",
            value: "Edit locations",
            comment: ""
        )
        navigationController.pushViewController(controller, animated: true)
    }
}

extension EditLocationsCoordinator: AddLocationsViewControllerDelegate {
    func didUpdateSelectedLocations(locations: [RelayLocation]) {
        customList.locations = locations
    }

    func didOnBack() {
        didFinish?(self, customList)
    }
}
