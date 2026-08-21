// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import Routing
import UIKit

class SetupAccountCompletedCoordinator: Coordinator, Presenting {
    private let navigationController: RootContainerViewController
    private var viewController: SetupAccountCompletedController?

    var didFinish: ((SetupAccountCompletedCoordinator) -> Void)?

    var presentationContext: UIViewController {
        viewController ?? navigationController
    }

    init(navigationController: RootContainerViewController) {
        self.navigationController = navigationController
    }

    func start(animated: Bool) {
        let controller = SetupAccountCompletedController()
        controller.delegate = self

        viewController = controller

        navigationController.pushViewController(controller, animated: animated)
    }
}

extension SetupAccountCompletedCoordinator: @preconcurrency SetupAccountCompletedControllerDelegate {
    func didRequestToSeePrivacy(controller: SetupAccountCompletedController) {
        presentChild(
            SafariCoordinator(
                url: ApplicationConfiguration.privacyGuidesURL()),
            animated: true)
    }

    func didRequestToStartTheApp(controller: SetupAccountCompletedController) {
        didFinish?(self)
    }
}
