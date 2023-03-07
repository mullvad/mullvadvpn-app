//
//  RevokedCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 07/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class RevokedCoordinator: Coordinator {
    let navigationController: RootContainerViewController
    private let tunnelManager: TunnelManager

    var didFinish: ((RevokedCoordinator) -> Void)?

    init(navigationController: RootContainerViewController, tunnelManager: TunnelManager) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
    }

    func start(animated: Bool) {
        let interactor = RevokedDeviceInteractor(tunnelManager: tunnelManager)
        let controller = RevokedDeviceViewController(interactor: interactor)

        controller.didFinish = { [weak self] in
            guard let self = self else { return }

            self.didFinish?(self)
        }

        navigationController.pushViewController(controller, animated: animated)
    }
}
