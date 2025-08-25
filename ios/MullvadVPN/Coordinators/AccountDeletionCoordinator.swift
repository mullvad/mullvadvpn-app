//
//  AccountDeletionCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Routing
import SwiftUI

final class AccountDeletionCoordinator: Coordinator, Presentable {
    private let navigationController: UINavigationController
    private let interactor: AccountDeletionInteractor

    var didConclude: (@MainActor (AccountDeletionCoordinator, Bool) -> Void)?

    var presentedViewController: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        interactor: AccountDeletionInteractor
    ) {
        self.navigationController = navigationController
        self.interactor = interactor
    }

    func start() {
        navigationController.navigationBar.isHidden = true
        let viewModel = interactor.makeViewModel(onConclusion: self.onConclusion)
        let viewController = UIHostingController(rootView: AccountDeletionView(viewModel: viewModel))
        navigationController.pushViewController(viewController, animated: true)
    }

    private func onConclusion(_ succeeded: Bool) {
        didConclude?(self, succeeded)
    }
}
