// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadLogging
import MullvadSettings
import Routing
import SwiftUI
import UIKit

final class NotificationSettingsCoordinator: Coordinator, Presentable, SettingsChildCoordinator {
    private let viewModel: NotificationSettingsViewModel
    private var navigationController: UINavigationController?
    var didUpdateNotificationSettings: ((NotificationSettingsCoordinator, NotificationSettings) -> Void)?

    var presentedViewController: UIViewController {
        navigationController!
    }

    init(
        navigationController: UINavigationController,
        viewModel: NotificationSettingsViewModel
    ) {
        self.viewModel = viewModel
        self.navigationController = navigationController
    }

    func start(animated: Bool) {
        var view = NotificationSettingsView(viewModel: viewModel)
        view.didUpdateNotificationSettings = { [weak self] notificationSettings in
            guard let self else { return }
            didUpdateNotificationSettings?(self, notificationSettings)
        }

        let viewController = UIHostingController(rootView: view)
        viewController.view.setAccessibilityIdentifier(.notificationSettingsView)
        viewController.navigationItem.title = NSLocalizedString("Notifications", comment: "")

        navigationController?.pushViewController(viewController, animated: animated)
    }
}
