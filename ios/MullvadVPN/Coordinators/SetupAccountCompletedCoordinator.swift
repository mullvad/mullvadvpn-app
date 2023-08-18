//
//  SetupAccountCompletedCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-03.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

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

extension SetupAccountCompletedCoordinator: SetupAccountCompletedControllerDelegate {
    func didRequestToSeePrivacy(controller: SetupAccountCompletedController) {
        presentChild(SafariCoordinator(url: ApplicationConfiguration.privacyGuidesURL), animated: true)
    }

    func didRequestToStartTheApp(controller: SetupAccountCompletedController) {
        didFinish?(self)
    }
}
