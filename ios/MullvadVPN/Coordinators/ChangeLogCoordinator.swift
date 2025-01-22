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

enum ChangeLogPresentationRoute {
    case settings, inAppNotification
}

final class ChangeLogCoordinator: Coordinator, Presentable, SettingsChildCoordinator {
    private let sourcePresentationRoute: ChangeLogPresentationRoute
    private let viewModel: ChangeLogViewModel
    private var navigationController: UINavigationController?

    var presentedViewController: UIViewController {
        navigationController!
    }

    init(
        sourcePresentationRoute: ChangeLogPresentationRoute,
        navigationController: UINavigationController,
        viewModel: ChangeLogViewModel
    ) {
        self.sourcePresentationRoute = sourcePresentationRoute
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

        switch sourcePresentationRoute {
        case .inAppNotification:
            let barButtonItem = UIBarButtonItem(
                title: NSLocalizedString(
                    "CHANGELOG_NAVIGATION_DONE_BUTTON",
                    tableName: "Changelog",
                    value: "Done",
                    comment: ""
                ),
                primaryAction: UIAction { _ in
                    self.dismiss(animated: true)
                }
            )
            barButtonItem.style = .done
            changeLogViewController.navigationItem.rightBarButtonItem = barButtonItem
            fallthrough
        case .settings:
            changeLogViewController.navigationItem.largeTitleDisplayMode = .always
            navigationController?.navigationBar.prefersLargeTitles = true
        }

        navigationController?.pushViewController(changeLogViewController, animated: animated)
    }
}
