// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import UIKit

class LaunchViewController: UIViewController {
    private let appResetManager: AppResetManager

    var onAppReady: (() -> Void)?

    init(
        launchArguments: LaunchArguments,
        tunnelManager: TunnelManager,
        settingsManager: SettingsManager
    ) {
        self.appResetManager = AppResetManager(
            launchArguments: launchArguments,
            tunnelManager: tunnelManager,
            settingsManager: settingsManager)
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

    override func viewDidLoad() {
        super.viewDidLoad()
        appResetManager.start()
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
