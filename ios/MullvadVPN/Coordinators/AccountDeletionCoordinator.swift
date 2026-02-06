//
//  AccountDeletionCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
