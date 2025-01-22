//
//  ChangeLogCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 24/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import Routing
import SwiftUI
import UIKit

final class ChangeLogCoordinator: Coordinator, Presentable, SettingsChildCoordinator {
    private let route: AppRoute
    private let viewModel: ChangeLogViewModel
    private var navigationController: UINavigationController?
    var didFinish: ((ChangeLogCoordinator) -> Void)?

    var presentedViewController: UIViewController {
        navigationController!
    }

    init(
        route: AppRoute,
        navigationController: UINavigationController,
        viewModel: ChangeLogViewModel
    ) {
        self.route = route
        self.viewModel = viewModel
        self.navigationController = navigationController
    }

    func start(animated: Bool) {
        let changeLogViewController = UIHostingController(rootView: ChangeLogView(viewModel: viewModel))
        changeLogViewController.view.setAccessibilityIdentifier(.changeLogAlert)
        changeLogViewController.navigationItem.title = NSLocalizedString(
            "whats_new_title",
            tableName: "Changelog",
            value: "What's new",
            comment: ""
        )

        switch route {
        case .changelog:
            let barButtonItem = UIBarButtonItem(
                title: NSLocalizedString(
                    "CHANGELOG_NAVIGATION_DONE_BUTTON",
                    tableName: "Changelog",
                    value: "Done",
                    comment: ""
                ),
                primaryAction: UIAction { [weak self] _ in
                    guard let self else { return }
                    didFinish?(self)
                }
            )
            barButtonItem.style = .done
            changeLogViewController.navigationItem.rightBarButtonItem = barButtonItem
            fallthrough
        case .settings:
            changeLogViewController.navigationItem.largeTitleDisplayMode = .always
            navigationController?.navigationBar.prefersLargeTitles = true
        default: break
        }

        navigationController?.pushViewController(changeLogViewController, animated: animated)
    }
}
