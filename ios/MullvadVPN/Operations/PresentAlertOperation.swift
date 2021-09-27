//
//  PresentAlertOperation.swift
//  PresentAlertOperation
//
//  Created by pronebird on 06/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class PresentAlertOperation: AsyncOperation {
    private let alertController: UIAlertController
    private let presentingController: UIViewController
    private let presentCompletion: (() -> Void)?

    init(alertController: UIAlertController, presentingController: UIViewController, presentCompletion: (() -> Void)? = nil) {
        self.alertController = alertController
        self.presentingController = presentingController
        self.presentCompletion = presentCompletion

        super.init()
    }

    override func cancel() {
        DispatchQueue.main.async {
            // Guard against executing cancellation more than once.
            guard !self.isCancelled else { return }

            // Call super implementation to toggle isCancelled flag
            super.cancel()

            // Guard against trying to dismiss the alert when operation hasn't started yet.
            guard self.isExecuting else { return }

            // Guard against dismissing controller during transition.
            if !self.alertController.isBeingPresented && !self.alertController.isBeingDismissed {
                self.dismissAndFinish()
            }
        }
    }

    override func main() {
        DispatchQueue.main.async {
            guard !self.isCancelled else {
                self.finish()
                return
            }

            NotificationCenter.default.addObserver(
                self,
                selector: #selector(self.alertControllerDidDismiss(_:)),
                name: AlertPresenter.alertControllerDidDismissNotification,
                object: self.alertController
            )

            self.presentingController.present(self.alertController, animated: true) {
                self.presentCompletion?()

                // Alert operation was cancelled during transition?
                if self.isCancelled {
                    self.dismissAndFinish()
                }
            }
        }
    }

    private func dismissAndFinish() {
        NotificationCenter.default.removeObserver(
            self,
            name: AlertPresenter.alertControllerDidDismissNotification,
            object: self.alertController
        )

        alertController.dismiss(animated: false) {
            self.finish()
        }
    }

    @objc private func alertControllerDidDismiss(_ note: Notification) {
        finish()
    }
}
