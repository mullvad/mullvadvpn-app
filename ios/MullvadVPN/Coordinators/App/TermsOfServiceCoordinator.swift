//
//  TermsOfServiceCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 29/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class TermsOfServiceCoordinator: Coordinator {
    private let navigationController: RootContainerViewController

    var didFinish: ((TermsOfServiceCoordinator) -> Void)?

    init(navigationController: RootContainerViewController) {
        self.navigationController = navigationController
    }

    func start() {
        let controller = TermsOfServiceViewController()

        controller.completionHandler = { [weak self] controller in
            guard let self = self else { return }

            TermsOfService.setAgreed()

            self.didFinish?(self)
        }

        navigationController.pushViewController(controller, animated: false)
    }
}
