//
//  PresentAlertOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 06/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Operations
import UIKit

public final class PresentAlertOperation: AsyncOperation {
    private let alertController: UIAlertController
    private let presentingController: UIViewController
    private let presentCompletion: (() -> Void)?

    public init(
        alertController: UIAlertController,
        presentingController: UIViewController,
        presentCompletion: (() -> Void)? = nil
    ) {
        self.alertController = alertController
        self.presentingController = presentingController
        self.presentCompletion = presentCompletion

        super.init(dispatchQueue: .main)
    }

    override public func operationDidCancel() {
        // Guard against trying to dismiss the alert when operation hasn't started yet.
        guard isExecuting else { return }

        // Guard against dismissing controller during transition.
        if !alertController.isBeingPresented, !alertController.isBeingDismissed {
            dismissAndFinish()
        }
    }

    override public func main() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(alertControllerDidDismiss(_:)),
            name: AlertPresenter.alertControllerDidDismissNotification,
            object: alertController
        )

        presentingController.present(alertController, animated: true) {
            self.presentCompletion?()

            // Alert operation was cancelled during transition?
            if self.isCancelled {
                self.dismissAndFinish()
            }
        }
    }

    private func dismissAndFinish() {
        NotificationCenter.default.removeObserver(
            self,
            name: AlertPresenter.alertControllerDidDismissNotification,
            object: alertController
        )

        alertController.dismiss(animated: false) {
            self.finish()
        }
    }

    @objc private func alertControllerDidDismiss(_ note: Notification) {
        finish()
    }
}
