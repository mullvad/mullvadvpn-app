//
//  PaymentErrorPresenter.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-30.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import UIKit

class PaymentAlertPresenter {
    private let presentationController: UIViewController
    private let alertPresenter: AlertPresenter

    init(presentationController: UIViewController, alertPresenter: AlertPresenter) {
        self.presentationController = presentationController
        self.alertPresenter = alertPresenter
    }

    func showAlertForError(
        _ error: StorePaymentManagerError,
        context: REST.CreateApplePaymentResponse.Context,
        completion: (() -> Void)? = nil
    ) {
        let alertController = CustomAlertViewController(
            title: context.errorTitle,
            message: error.displayErrorDescription
        )

        alertController.addAction(
            title: okButtonTextForKey("PAYMENT_ERROR_ALERT_OK_ACTION"),
            style: .default,
            handler: {
                completion?()
            }
        )

        alertPresenter.enqueue(alertController, presentingController: presentationController)
    }

    func showAlertForResponse(
        _ response: REST.CreateApplePaymentResponse,
        context: REST.CreateApplePaymentResponse.Context,
        completion: (() -> Void)? = nil
    ) {
        guard case .noTimeAdded = response else {
            completion?()
            return
        }

        let alertController = CustomAlertViewController(
            title: response.alertTitle(context: context),
            message: response.alertMessage(context: context)
        )

        alertController.addAction(
            title: okButtonTextForKey("PAYMENT_RESPONSE_ALERT_OK_ACTION"),
            style: .default,
            handler: {
                completion?()
            }
        )

        alertPresenter.enqueue(alertController, presentingController: presentationController)
    }

    private func okButtonTextForKey(_ key: String) -> String {
        NSLocalizedString(
            key,
            tableName: "Payment",
            value: "Got it!",
            comment: ""
        )
    }
}
