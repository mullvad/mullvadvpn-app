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
    private let presentation: AlertPresentation

    var didFinish: (() -> Void)?

    var presentedViewController: UIViewController {
        return alertController!
    }

    init(presentation: AlertPresentation) {
        self.presentation = presentation
    }

    func start() {
        let alertController = AlertViewController(
            header: presentation.header,
            title: presentation.title,
            message: presentation.message,
            icon: presentation.icon
        )

        self.alertController = alertController

        alertController.onDismiss = { [weak self] in
            self?.didFinish?()
        }

        presentation.buttons.forEach { action in
            alertController.addAction(
                title: action.title,
                style: action.style,
                accessibilityId: action.accessibilityID,
                handler: action.handler
            )
        }
    }
}
