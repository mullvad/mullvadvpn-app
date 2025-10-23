//
//  AddLocationsCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-03-04.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class AddLocationsCoordinator: Coordinator, Presentable, Presenting {
    private let navigationController: UINavigationController
    private let nodes: [LocationNode]
    private var subject: CurrentValueSubject<CustomListViewModel, Never>

    var didFinish: ((AddLocationsCoordinator) -> Void)?

    var presentedViewController: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        nodes: [LocationNode],
        subject: CurrentValueSubject<CustomListViewModel, Never>
    ) {
        self.navigationController = navigationController
        self.nodes = nodes
        self.subject = subject
    }

    func start() {
        let controller = AddLocationsViewController(
            allLocationsNodes: nodes,
            subject: subject
        )
        controller.delegate = self

        controller.navigationItem.title = NSLocalizedString("Locations", comment: "")

        navigationController.pushViewController(controller, animated: true)
    }
}

extension AddLocationsCoordinator: @preconcurrency AddLocationsViewControllerDelegate {
    func didBack() {
        didFinish?(self)
    }
}
