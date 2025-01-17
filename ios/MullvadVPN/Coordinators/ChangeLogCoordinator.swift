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
    private var navigationController: UINavigationController?
    private let viewModel: ChangeLogViewModel

    var presentedViewController: UIViewController {
        return navigationController!
    }

    init(navigationController: UINavigationController, viewModel: ChangeLogViewModel) {
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
        changeLogViewController.navigationItem.largeTitleDisplayMode = .always
        navigationController?.navigationBar.prefersLargeTitles = true
        navigationController?.pushViewController(changeLogViewController, animated: animated)
    }
}
