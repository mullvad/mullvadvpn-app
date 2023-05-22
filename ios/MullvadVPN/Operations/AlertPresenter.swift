//
//  AlertPresenter.swift
//  MullvadVPN
//
//  Created by pronebird on 04/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Operations
import UIKit

final class AlertPresenter {
    static let alertControllerDidDismissNotification = Notification
        .Name("CustomAlertControllerDidDismiss")

    private let operationQueue = AsyncOperationQueue.makeSerial()

    func enqueue(
        _ alertController: CustomAlertViewController,
        presentingController: UIViewController,
        presentCompletion: (() -> Void)? = nil
    ) {
        let operation = PresentAlertOperation(
            alertController: alertController,
            presentingController: presentingController,
            presentCompletion: presentCompletion
        )

        alertController.didDismiss = { [weak self] in
            self?.onAlertControllerDismiss(alertController)
        }

        operationQueue.addOperation(operation)
    }

    func cancelAll() {
        operationQueue.cancelAllOperations()
    }

    private func onAlertControllerDismiss(_ alertController: CustomAlertViewController) {
        if alertController.presentingViewController == nil {
            NotificationCenter.default.post(
                name: AlertPresenter.alertControllerDidDismissNotification,
                object: alertController
            )
        }
    }
}
