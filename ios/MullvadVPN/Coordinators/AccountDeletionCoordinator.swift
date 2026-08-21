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
import SwiftUI

final class AccountDeletionCoordinator: Coordinator, Presentable {
    private let navigationController: UINavigationController
    private let tunnelManager: TunnelManager

    var didConclude: (@MainActor (AccountDeletionCoordinator, Bool) -> Void)?

    var presentedViewController: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        tunnelManager: TunnelManager
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
    }

    func start() {
        navigationController.navigationBar.isHidden = true
        let viewModel = AccountDeletionViewModel(
            tunnelManager: tunnelManager,
            onConclusion: self.onConclusion(_:)
        )
        let viewController = UIHostingController(rootView: AccountDeletionView(viewModel: viewModel))
        viewController.view.setAccessibilityIdentifier(.deleteAccountView)
        navigationController.pushViewController(viewController, animated: true)
    }

    private func onConclusion(_ succeeded: Bool) {
        didConclude?(self, succeeded)
    }
}
