//
//  EditLocationsCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-03-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import MullvadTypes
import Routing
import UIKit

@MainActor
class EditLocationsCoordinator: Coordinator, Presentable, Presenting {
    private let navigationController: UINavigationController
    private let nodes: [LocationNode]
    private var subject: CurrentValueSubject<CustomListViewModel, Never>

    nonisolated(unsafe) var didFinish: (@Sendable (EditLocationsCoordinator) -> Void)?

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

extension EditLocationsCoordinator: AddLocationsViewControllerDelegate {
    nonisolated func didBack() {
        didFinish?(self)
    }
}
