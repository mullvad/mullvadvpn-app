//
//  RevokedCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 07/03/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Routing
import SwiftUI
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
        let viewModel = RevokedDeviceViewModel(interactor: interactor)

        var view = RevokedDeviceView(viewModel: viewModel)
        view.onLogout = { [weak self] in
            guard let self else { return }
            didFinish?(self)
        }

        let controller = UIHostingController(rootView: view)
        controller.view.setAccessibilityIdentifier(.revokedDeviceView)

        navigationController.pushViewController(controller, animated: animated)
    }
}
