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
        alertController = AlertViewController(presentation: presentation)

        alertController?.onDismiss = { [weak self] in
            self?.didFinish?()
        }
    }
}
