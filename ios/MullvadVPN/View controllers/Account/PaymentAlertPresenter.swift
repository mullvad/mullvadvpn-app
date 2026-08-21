// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadREST
import Routing

@MainActor
struct PaymentAlertPresenter {
    let alertContext: any Presenting

    func showAlertForOutcome(
        _ outcome: StorePaymentOutcome,
        context: StorePaymentOutcome.Context,
        completion: (@MainActor @Sendable () -> Void)? = nil
    ) {
        let presentation = AlertPresentation(
            id: "payment-outcome-alert",
            title: context.alertTitle,
            message: outcome.alertMessage(for: context),
            buttons: [
                AlertAction(
                    title: NSLocalizedString("Got it!", comment: ""),
                    style: .default,
                    handler: {
                        completion?()
                    }
                )
            ]
        )

        let presenter = AlertPresenter(context: alertContext)
        presenter.showAlert(presentation: presentation, animated: true)
    }

    func showAlertForError(
        _ error: StorePaymentError,
        context: StorePaymentOutcome.Context,
        completion: (@MainActor @Sendable () -> Void)? = nil
    ) {
        let presentation = AlertPresentation(
            id: "payment-error-alert",
            title: context.errorTitle,
            message: error.description,
            buttons: [
                AlertAction(
                    title: NSLocalizedString("Got it!", comment: ""),
                    style: .default,
                    handler: {
                        completion?()
                    }
                )
            ]
        )

        let presenter = AlertPresenter(context: alertContext)
        presenter.showAlert(presentation: presentation, animated: true)
    }

    func showAlertForRefund(completion: (@MainActor @Sendable () -> Void)? = nil) {
        let presentation = AlertPresentation(
            id: "payment-refund-alert",
            title: NSLocalizedString("Refund successful", comment: ""),
            message: NSLocalizedString("Your purchase was successfully refunded.", comment: ""),
            buttons: [
                AlertAction(
                    title: NSLocalizedString("Got it!", comment: ""),
                    style: .default,
                    handler: {
                        completion?()
                    }
                )
            ]
        )

        let presenter = AlertPresenter(context: alertContext)
        presenter.showAlert(presentation: presentation, animated: true)
    }

    func showAlertForRefundError(
        _ error: any Error,
        context: StorePaymentOutcome.Context,
        completion: (() -> Void)? = nil
    ) {
        let presentation = AlertPresentation(
            id: "payment-refund-error-alert",
            title: context.errorTitle,
            message: "\(error)",
            buttons: [
                AlertAction(
                    title: NSLocalizedString("Got it!", comment: ""),
                    style: .default,
                    handler: {
                        completion?()
                    }
                )
            ]
        )

        let presenter = AlertPresenter(context: alertContext)
        presenter.showAlert(presentation: presentation, animated: true)
    }
}
