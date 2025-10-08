//
//  PaymentErrorPresenter.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-30.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import Routing

@MainActor
struct PaymentAlertPresenter {
    let alertContext: any Presenting

    // MARK: StoreKit 2 flow

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

    // MARK: Legacy StoreKit flow

    func showAlertForError(
        _ error: LegacyStorePaymentManagerError,
        context: StorePaymentOutcome.Context,
        completion: (@MainActor @Sendable () -> Void)? = nil
    ) {
        let presentation = AlertPresentation(
            id: "payment-error-alert",
            title: context.errorTitle,
            message: error.displayErrorDescription,
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

    // MARK: StoreKit 2 refunds

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
