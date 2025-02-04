//
//  AccountDeletionCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Routing
import UIKit

final class AccountDeletionCoordinator: Coordinator, Presentable {
    private let navigationController: UINavigationController
    private let interactor: AccountDeletionInteractor

    var didCancel: (@MainActor (AccountDeletionCoordinator) -> Void)?
    var didFinish: (@MainActor (AccountDeletionCoordinator) -> Void)?

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
        let viewController = AccountDeletionViewController(interactor: interactor)
        viewController.delegate = self
        navigationController.pushViewController(viewController, animated: true)
    }
}

extension AccountDeletionCoordinator: @preconcurrency AccountDeletionViewControllerDelegate {
    func deleteAccountDidSucceed(controller: AccountDeletionViewController) {
        didFinish?(self)
    }

    func deleteAccountDidCancel(controller: AccountDeletionViewController) {
        didCancel?(self)
    }
}
