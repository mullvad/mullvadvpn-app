// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
