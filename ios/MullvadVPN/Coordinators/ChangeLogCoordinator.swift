//
//  ChangeLogCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 24/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import Routing
import UIKit

final class ChangeLogCoordinator: Coordinator, Presentable {
    private var alertController: AlertViewController?
    private let interactor: ChangeLogInteractor
    var didFinish: ((ChangeLogCoordinator) -> Void)?

    var presentedViewController: UIViewController {
        return alertController!
    }

    init(interactor: ChangeLogInteractor) {
        self.interactor = interactor
    }

    func start() {
        let alertController = AlertViewController(
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
            style: .default,
            handler: { [weak self] in
                guard let self else { return }
                didFinish?(self)
            }
        )
        self.alertController = alertController
    }
}
