//
//  PaymentErrorPresenter.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-30.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import Routing

struct PaymentAlertPresenter {
    let coordinator: Coordinator

    func showAlertForError(
        _ error: StorePaymentManagerError,
        context: REST.CreateApplePaymentResponse.Context,
        completion: (() -> Void)? = nil
    ) {
        let presentation = AlertPresentation(
            title: context.errorTitle,
            message: error.displayErrorDescription,
            buttons: [
                AlertAction(
                    title: okButtonTextForKey("PAYMENT_ERROR_ALERT_OK_ACTION"),
                    style: .default,
                    handler: {
                        completion?()
                    }
                ),
            ]
        )

        coordinator.applicationRouter?.present(.alert(presentation), animated: true)
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

        let presentation = AlertPresentation(
            title: response.alertTitle(context: context),
            message: response.alertMessage(context: context),
            buttons: [
                AlertAction(
                    title: okButtonTextForKey("PAYMENT_RESPONSE_ALERT_OK_ACTION"),
                    style: .default,
                    handler: {
                        completion?()
                    }
                ),
            ]
        )

        coordinator.applicationRouter?.present(.alert(presentation), animated: true)
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
