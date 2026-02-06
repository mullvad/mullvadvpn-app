//
//  NotificationSettingsCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-01-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
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
