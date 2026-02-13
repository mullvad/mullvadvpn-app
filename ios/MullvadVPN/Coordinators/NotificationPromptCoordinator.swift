//
//  NotificationPromptCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-01-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import Routing
import SwiftUI
import UIKit

final class NotificationPromptCoordinator: Coordinator, Presentable {
    let navigationController: UINavigationController
    var didConclude: (@MainActor (NotificationPromptCoordinator) -> Void)? = nil

    var presentedViewController: UIViewController {
        return navigationController
    }

    init(navigationController: UINavigationController) {
        self.navigationController = navigationController
    }

    func start(animated: Bool) {
        let viewModel = NotificationPromptViewModel()
        let view = NotificationPromptView(
            viewModel: viewModel,
            didConclude: { [weak self] _ in
                guard let self else { return }
                didConclude?(self)
            })
        let viewController = UIHostingController(rootView: view)
        viewController.view.setAccessibilityIdentifier(.notificationPromptView)
        navigationController.isNavigationBarHidden = true
        navigationController.pushViewController(viewController, animated: animated)
    }
}
