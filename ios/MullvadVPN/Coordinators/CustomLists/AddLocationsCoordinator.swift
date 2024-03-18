//
//  AddLocationsCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-03-04.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class AddLocationsCoordinator: Coordinator, Presentable, Presenting {
    private let navigationController: UINavigationController
    private let nodes: [LocationNode]
    private var customList: CustomList

    var didFinish: ((CustomList) -> Void)?

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
            "ADD_LOCATIONS_NAVIGATION_TITLE",
            tableName: "AddLocations",
            value: "Add locations",
            comment: ""
        )

        navigationController.pushViewController(controller, animated: false)
        navigationController.delegate = self
    }
}

extension AddLocationsCoordinator: AddLocationsViewControllerDelegate {
    func didUpdateSelectedLocations(locations: [RelayLocation]) {
        self.customList.locations = locations
    }
}

extension AddLocationsCoordinator: UINavigationControllerDelegate {
    func navigationController(
        _ navigationController: UINavigationController,
        willShow viewController: UIViewController,
        animated: Bool
    ) {
        guard viewController is CustomListViewController else {
            return
        }
        didFinish?(customList)
    }
}
