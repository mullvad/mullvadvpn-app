//
//  AlertCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-08-23.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import Routing
import UIKit

final class AlertCoordinator: Coordinator, Presentable {
    private var alertController: AlertViewController?
    private let interactor: AlertInteractor

    var presentedViewController: UIViewController {
        return alertController!
    }

    init(interactor: AlertInteractor) {
        self.interactor = interactor
    }

    func start() {
        let alertController = AlertViewController(
            header: interactor.presentation.header,
            title: interactor.presentation.title,
            message: interactor.presentation.message
        )
        self.alertController = alertController

        interactor.presentation.buttons.forEach { action in
            alertController.addAction(title: action.title, style: action.style) { [weak self] in
                self?.interactor.presentation.onDismiss?(action)
            }
        }
    }
}
