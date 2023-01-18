//
//  OutOfTimeViewController.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-07-25.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import Operations
import StoreKit
import UIKit

protocol OutOfTimeViewControllerDelegate: AnyObject {
    func outOfTimeViewControllerDidBeginPayment(_ controller: OutOfTimeViewController)
    func outOfTimeViewControllerDidEndPayment(_ controller: OutOfTimeViewController)
}

class OutOfTimeViewController: UIViewController, RootContainment {
    weak var delegate: OutOfTimeViewControllerDelegate?

    private let interactor: OutOfTimeInteractor
    private let alertPresenter = AlertPresenter()

    private var productState: ProductState = .none {
        didSet {
            applyViewState()
        }
    }

    private var paymentState: PaymentState = .none {
        didSet {
            applyViewState()
            notifyDelegate(oldValue)
        }
    }

    private lazy var contentView = OutOfTimeContentView()

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        let tunnelState = interactor.tunnelStatus.state

        return HeaderBarPresentation(
            style: tunnelState.isSecured ? .secured : .unsecured,
            showsDivider: false
        )
    }

    var prefersHeaderBarHidden: Bool {
        return false
    }

    init(interactor: OutOfTimeInteractor) {
        self.interactor = interactor

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])

        contentView.disconnectButton.addTarget(
            self,
            action: #selector(handleDisconnect(_:)),
            for: .touchUpInside
        )
        contentView.purchaseButton.addTarget(
            self,
            action: #selector(doPurchase),
            for: .touchUpInside
        )
        contentView.restoreButton.addTarget(
            self,
            action: #selector(restorePurchases),
            for: .touchUpInside
        )

        interactor.didReceivePaymentEvent = { [weak self] event in
            self?.didReceivePaymentEvent(event)
        }

        interactor.didReceiveTunnelStatus = { [weak self] tunnelStatus in
            self?.setNeedsHeaderBarStyleAppearanceUpdate()
        }

        if StorePaymentManager.canMakePayments {
            requestStoreProducts()
        } else {
            productState = .cannotMakePurchases
        }
    }

    // MARK: - Private

    private func requestStoreProducts() {
        let productKind = StoreSubscription.thirtyDays

        productState = .fetching(productKind)

        _ = interactor.requestProducts(with: [productKind]) { [weak self] completion in
            let productState: ProductState = completion.value?.products.first
                .map { .received($0) } ?? .failed

            self?.productState = productState
        }
    }

    private func applyViewState() {
        let tunnelState = interactor.tunnelStatus.state
        let isInteractionEnabled = paymentState.allowsViewInteraction
        let purchaseButton = contentView.purchaseButton

        let isOutOfTime = interactor.deviceState.accountData.map { $0.expiry < Date() } ?? false

        purchaseButton.setTitle(productState.purchaseButtonTitle, for: .normal)
        contentView.purchaseButton.isLoading = productState.isFetching

        purchaseButton.isEnabled = productState.isReceived && isInteractionEnabled && !tunnelState
            .isSecured
        contentView.restoreButton.isEnabled = isInteractionEnabled
        contentView.disconnectButton.isEnabled = tunnelState.isSecured
        contentView.disconnectButton.alpha = tunnelState.isSecured ? 1 : 0

        if tunnelState.isSecured {
            contentView.setBodyLabelText(
                NSLocalizedString(
                    "OUT_OF_TIME_BODY_CONNECTED",
                    tableName: "OutOfTime",
                    value: "You have no more VPN time left on this account. To add more, you will need to disconnect and access the Internet with an unsecure connection.",
                    comment: ""
                )
            )
        } else {
            contentView.setBodyLabelText(
                NSLocalizedString(
                    "OUT_OF_TIME_BODY_DISCONNECTED",
                    tableName: "OutOfTime",
                    value: "You have no more VPN time left on this account. Either buy credit on our website or make an in-app purchase via the **Add 30 days time** button below.",
                    comment: ""
                )
            )
        }

        if !isInteractionEnabled {
            contentView.statusActivityView.state = .activity
        } else {
            contentView.statusActivityView.state = isOutOfTime ? .failure : .success
        }

        view.isUserInteractionEnabled = isInteractionEnabled
    }

    private func notifyDelegate(_ oldPaymentState: PaymentState) {
        switch (oldPaymentState, paymentState) {
        case (.none, .makingPayment), (.none, .restoringPurchases):
            delegate?.outOfTimeViewControllerDidBeginPayment(self)

        case (.makingPayment, .none), (.restoringPurchases, .none):
            delegate?.outOfTimeViewControllerDidEndPayment(self)

        default:
            break
        }
    }

    private func didReceivePaymentEvent(_ event: StorePaymentEvent) {
        guard case let .makingPayment(payment) = paymentState,
              payment == event.payment else { return }

        switch event {
        case .finished:
            break

        case let .failure(paymentFailure):
            switch paymentFailure.error {
            case .storePayment(SKError.paymentCancelled):
                break

            default:
                showPaymentErrorAlert(error: paymentFailure.error)
            }
        }

        paymentState = .none
    }

    private func showPaymentErrorAlert(error: StorePaymentManagerError) {
        let alertController = UIAlertController(
            title: NSLocalizedString(
                "CANNOT_COMPLETE_PURCHASE_ALERT_TITLE",
                tableName: "OutOfTime",
                value: "Cannot complete the purchase",
                comment: ""
            ),
            message: error.displayErrorDescription,
            preferredStyle: .alert
        )

        alertController.addAction(
            UIAlertAction(
                title: NSLocalizedString(
                    "CANNOT_COMPLETE_PURCHASE_ALERT_OK_ACTION",
                    tableName: "OutOfTime",
                    value: "OK",
                    comment: ""
                ), style: .cancel
            )
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    private func showRestorePurchasesErrorAlert(error: StorePaymentManagerError) {
        let alertController = UIAlertController(
            title: NSLocalizedString(
                "RESTORE_PURCHASES_FAILURE_ALERT_TITLE",
                tableName: "OutOfTime",
                value: "Cannot restore purchases",
                comment: ""
            ),
            message: error.displayErrorDescription,
            preferredStyle: .alert
        )

        alertController.addAction(
            UIAlertAction(title: NSLocalizedString(
                "RESTORE_PURCHASES_FAILURE_ALERT_OK_ACTION",
                tableName: "OutOfTime",
                value: "OK",
                comment: ""
            ), style: .cancel)
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    private func showAlertIfNoTimeAdded(
        with response: REST.CreateApplePaymentResponse,
        context: REST.CreateApplePaymentResponse.Context
    ) {
        guard case .noTimeAdded = response else { return }

        let alertController = UIAlertController(
            title: response.alertTitle(context: context),
            message: response.alertMessage(context: context),
            preferredStyle: .alert
        )

        alertController.addAction(
            UIAlertAction(
                title: NSLocalizedString(
                    "TIME_ADDED_ALERT_OK_ACTION",
                    tableName: "OutOfTime",
                    value: "OK",
                    comment: ""
                ),
                style: .cancel
            )
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    // MARK: - Actions

    @objc private func doPurchase() {
        guard case let .received(product) = productState,
              let accountData = interactor.deviceState.accountData
        else {
            return
        }

        let payment = SKPayment(product: product)
        interactor.addPayment(payment, for: accountData.number)

        paymentState = .makingPayment(payment)
    }

    @objc func restorePurchases() {
        guard let accountData = interactor.deviceState.accountData else {
            return
        }

        paymentState = .restoringPurchases

        _ = interactor.restorePurchases(for: accountData.number) { [weak self] completion in
            guard let self = self else { return }

            switch completion {
            case let .success(response):
                self.showAlertIfNoTimeAdded(with: response, context: .restoration)

            case let .failure(error):
                self.showRestorePurchasesErrorAlert(error: error)

            case .cancelled:
                break
            }

            self.paymentState = .none
        }
    }

    @objc private func handleDisconnect(_ sender: Any) {
        interactor.stopTunnel()
    }
}
