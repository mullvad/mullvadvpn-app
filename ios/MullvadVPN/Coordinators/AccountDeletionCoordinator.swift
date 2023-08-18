//
//  AccountDeletionCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Routing
import UIKit

final class AccountDeletionCoordinator: Coordinator, Presentable {
    private let navigationController: UINavigationController
    private let interactor: AccountDeletionInteractor

    var didCancel: ((AccountDeletionCoordinator) -> Void)?
    var didFinish: ((AccountDeletionCoordinator) -> Void)?

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

extension AccountDeletionCoordinator: AccountDeletionViewControllerDelegate {
    func deleteAccountDidSucceed(controller: AccountDeletionViewController) {
        didFinish?(self)
    }

    func deleteAccountDidCancel(controller: AccountDeletionViewController) {
        didCancel?(self)
    }
}
