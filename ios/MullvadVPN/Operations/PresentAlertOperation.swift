//
//  PresentAlertOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 06/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Operations
import UIKit

final class PresentAlertOperation: AsyncOperation {
    private let alertController: CustomAlertViewController
    private let presentingController: UIViewController
    private let presentCompletion: (() -> Void)?

    init(
        alertController: CustomAlertViewController,
        presentingController: UIViewController,
        presentCompletion: (() -> Void)? = nil
    ) {
        self.alertController = alertController
        self.presentingController = presentingController
        self.presentCompletion = presentCompletion

        super.init(dispatchQueue: .main)
    }

    override func operationDidCancel() {
        // Guard against trying to dismiss the alert when operation hasn't started yet.
        guard isExecuting else { return }

        // Guard against dismissing controller during transition.
        if !alertController.isBeingPresented, !alertController.isBeingDismissed {
            dismissAndFinish()
        }
    }

    override func main() {
        alertController.didDismiss = { [weak self] in
            guard let self else { return }
            finish()
        }

        presentingController.present(alertController, animated: true) {
            self.presentCompletion?()

            // Alert operation was cancelled during transition?
            if self.isCancelled {
                self.dismissAndFinish()
            }
        }
    }

    private func dismissAndFinish() {
        alertController.dismiss(animated: false) {
            self.finish()
        }
    }
}
