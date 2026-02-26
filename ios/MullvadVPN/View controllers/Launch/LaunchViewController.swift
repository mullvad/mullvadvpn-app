//
//  LaunchViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 18/11/2021.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

class LaunchViewController: UIViewController {
    private let appResetManager: AppResetManager

    var onAppReady: (() -> Void)?

    init(launchArguments: LaunchArguments, tunnelManager: TunnelManager) {
        self.appResetManager = AppResetManager(
            launchArguments: launchArguments, tunnelManager: tunnelManager)
        super.init(nibName: nil, bundle: nil)
        setupLaunchScreen()
        self.appResetManager.onAppReady = { [weak self] in
            self?.onAppReady?()
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    private func setupLaunchScreen() {
        let storyboard = UIStoryboard(name: "LaunchScreen", bundle: nil)

        guard let initialController = storyboard.instantiateInitialViewController() else {
            assertionFailure("LaunchScreen storyboard misconfigured")
            return
        }

        initialController.view.translatesAutoresizingMaskIntoConstraints = false

        addChild(initialController)
        view.addSubview(initialController.view)
        initialController.didMove(toParent: self)

        NSLayoutConstraint.activate([
            initialController.view.topAnchor.constraint(equalTo: view.topAnchor),
            initialController.view.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            initialController.view.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            initialController.view.trailingAnchor.constraint(equalTo: view.trailingAnchor),
        ])
    }
}
