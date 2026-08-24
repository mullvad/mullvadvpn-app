// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
