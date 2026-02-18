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
    private let tunnelManager: TunnelManager
    private var hasStartedBootstrap = false
    private var isFirstLaunch: Bool = true

    var onAppReady: (() -> Void)?

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        super.init(nibName: nil, bundle: nil)
        setupLaunchScreen()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        Task { [weak self] in
            await self?.bootstrap()
        }

    }

    private func bootstrap() async {
        guard let launchArguments = try? ProcessInfo.processInfo.decode(LaunchArguments.self) else {
            onAppReady?()
            return
        }

        if launchArguments.isResetAppAllowed {
            tunnelManager.stopTunnel()
            try? SettingsManager.writeSettings(LatestTunnelSettings())
            Task {
                await tunnelManager.unsetAccount()
                isFirstLaunch = false
                onAppReady?()
            }
        }
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

    private func prepareApp(launchArguments: LaunchArguments) async {
        if launchArguments.isResetAppAllowed {
            await tunnelManager.unsetAccount()
        }

        isFirstLaunch = false
    }

}
