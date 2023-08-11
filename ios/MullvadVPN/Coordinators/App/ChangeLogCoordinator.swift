//
//  ChangeLogCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 24/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import UIKit

final class ChangeLogCoordinator: Coordinator {
    private let navigationController: UIViewController
    private let interactor: ChangeLogInteractor
    var didFinish: ((ChangeLogCoordinator) -> Void)?

    var presentedViewController: UIViewController {
        return navigationController
    }

    init(
        navigationController: UIViewController,
        interactor: ChangeLogInteractor
    ) {
        self.navigationController = navigationController
        self.interactor = interactor
    }

    func start(animated: Bool) {
        let alertController = CustomAlertViewController(
            header: interactor.viewModel.header,
            title: interactor.viewModel.title,
            attributedMessage: interactor.viewModel.body
        )

        alertController.addAction(
            title: NSLocalizedString(
                "CHANGE_LOG_OK_ACTION",
                tableName: "Account",
                value: "Got it!",
                comment: ""
            ),
            style: .default
        )
        presentedViewController.present(alertController, animated: animated) {
            self.didFinish?(self)
        }
    }
}
