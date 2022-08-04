//
//  OutOfTimeViewController.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-07-25.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit
import UIKit

protocol OutOfTimeViewControllerDelegate: AnyObject {
    func outOfTimeViewControllerDidAddTime(_ controller: OutOfTimeViewController)
}

class OutOfTimeViewController: UIViewController {
    weak var delegate: OutOfTimeViewControllerDelegate?

    private var product: SKProduct?
    private var pendingPayment: SKPayment?
    private let alertPresenter = AlertPresenter()

    private lazy var contentView = OutOfTimeContentView()

    private lazy var purchaseButtonInteractionRestriction =
        UserInterfaceInteractionRestriction { [weak self] enableUserInteraction, _ in
            // Make sure to disable the button if the product is not loaded
            self?.contentView.purchaseButton.isEnabled = enableUserInteraction &&
                self?.product != nil &&
                AppStorePaymentManager.canMakePayments
        }

    private lazy var viewControllerInteractionRestriction =
        UserInterfaceInteractionRestriction { [weak self] enableUserInteraction, _ in
            self?.setEnableUserInteraction(enableUserInteraction, animated: true)
        }

    private lazy var compoundInteractionRestriction = CompoundUserInterfaceInteractionRestriction(
        restrictions: [
            purchaseButtonInteractionRestriction,
            viewControllerInteractionRestriction,
        ]
    )

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    override func viewDidLoad() {
        setUpContentView()
        setUpButtonTargets()
        setUpInAppPurchases()
        addObservers()
    }
}

// MARK: - Private Functions

private extension OutOfTimeViewController {
    func setUpContentView() {
        view.addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }

    func setUpButtonTargets() {
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
    }

    func addObservers() {
        AppStorePaymentManager.shared.addPaymentObserver(self)
    }

    private func setEnableUserInteraction(_ enableUserInteraction: Bool, animated _: Bool) {
        // Disable all buttons
        [contentView.purchaseButton, contentView.redeemButton, contentView.restoreButton]
            .forEach { button in
                button?.isEnabled = enableUserInteraction
            }

        // Disable any interaction within the view
        view.isUserInteractionEnabled = enableUserInteraction
    }
}

// MARK: - In App Purchases

private extension OutOfTimeViewController {
    func setUpInAppPurchases() {
        if AppStorePaymentManager.canMakePayments {
            requestStoreProducts()
        } else {
            setPaymentsRestricted()
        }
    }

    func requestStoreProducts() {
        let inAppPurchase = AppStoreSubscription.thirtyDays

        contentView.purchaseButton.setTitle(inAppPurchase.localizedTitle, for: .normal)
        contentView.purchaseButton.isLoading = true

        purchaseButtonInteractionRestriction.increase(animated: true)

        _ = AppStorePaymentManager.shared
            .requestProducts(with: [inAppPurchase]) { [weak self] completion in
                guard let self = self else { return }

                switch completion {
                case let .success(response):
                    if let product = response.products.first {
                        self.setProduct(product, animated: true)
                    }

                case let .failure(error):
                    self.didFailLoadingProducts(with: error)

                case .cancelled:
                    break
                }

                self.contentView.purchaseButton.isLoading = false
                self.purchaseButtonInteractionRestriction.decrease(animated: true)
            }
    }

    func setProduct(_ product: SKProduct, animated _: Bool) {
        self.product = product

        let localizedTitle = product.customLocalizedTitle ?? ""
        let localizedPrice = product.localizedPrice ?? ""

        let format = NSLocalizedString(
            "PURCHASE_BUTTON_TITLE_FORMAT",
            tableName: "OutOfTime",
            value: "%1$@ (%2$@)",
            comment: ""
        )
        let title = String(format: format, localizedTitle, localizedPrice)

        contentView.purchaseButton.setTitle(title, for: .normal)
    }

    func didFailLoadingProducts(with _: Error) {
        let title = NSLocalizedString(
            "PURCHASE_BUTTON_CANNOT_CONNECT_TO_APPSTORE_LABEL",
            tableName: "OutOfTime",
            value: "Cannot connect to AppStore",
            comment: ""
        )

        contentView.purchaseButton.setTitle(title, for: .normal)
    }

    func setPaymentsRestricted() {
        let title = NSLocalizedString(
            "PURCHASE_BUTTON_PAYMENTS_RESTRICTED_LABEL",
            tableName: "OutOfTime",
            value: "Payments restricted",
            comment: ""
        )

        contentView.purchaseButton.setTitle(title, for: .normal)
        contentView.purchaseButton.isEnabled = false
    }

    @objc func doPurchase() {
        guard let accountData = TunnelManager.shared.deviceState.accountData,
              let product = product else { return }

        let payment = SKPayment(product: product)

        pendingPayment = payment
        compoundInteractionRestriction.increase(animated: true)

        AppStorePaymentManager.shared.addPayment(payment, for: accountData.number)
    }

    @objc private func restorePurchases() {
        guard let accountNumber = TunnelManager.shared.deviceState.accountData?.number
        else { return }

        compoundInteractionRestriction.increase(animated: true)

        _ = AppStorePaymentManager.shared.restorePurchases(for: accountNumber) { completion in
            switch completion {
            case let .success(response):
                self.showTimeAddedConfirmationAlert(with: response, context: .restoration)

            case let .failure(error):
                let alertController = UIAlertController(
                    title: NSLocalizedString(
                        "RESTORE_PURCHASES_FAILURE_ALERT_TITLE",
                        tableName: "OutOfTime",
                        value: "Cannot restore purchases",
                        comment: ""
                    ),
                    message: error.errorChainDescription,
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
                self.alertPresenter.enqueue(alertController, presentingController: self)

            case .cancelled:
                break
            }

            self.compoundInteractionRestriction.decrease(animated: true)
        }
    }

    private func showTimeAddedConfirmationAlert(
        with response: REST.CreateApplePaymentResponse,
        context: REST.CreateApplePaymentResponse.Context
    ) {
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
            ) { _ in
                self.didAddMoreTime()
            }
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    @objc func didAddMoreTime() {
        delegate?.outOfTimeViewControllerDidAddTime(self)
    }
}

// MARK: - AppStorePaymentObserver

extension OutOfTimeViewController: AppStorePaymentObserver {
    func appStorePaymentManager(
        _: AppStorePaymentManager,
        transaction _: SKPaymentTransaction?,
        payment: SKPayment,
        accountToken _: String?,
        didFailWithError error: AppStorePaymentManager.Error
    ) {
        let alertController = UIAlertController(
            title: NSLocalizedString(
                "CANNOT_COMPLETE_PURCHASE_ALERT_TITLE",
                tableName: "OutOfTime",
                value: "Cannot complete the purchase",
                comment: ""
            ),
            message: error.errorChainDescription,
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

        if payment == pendingPayment {
            compoundInteractionRestriction.decrease(animated: true)
        }
    }

    func appStorePaymentManager(
        _: AppStorePaymentManager,
        transaction: SKPaymentTransaction,
        accountToken _: String,
        didFinishWithResponse _: REST.CreateApplePaymentResponse
    ) {
        if transaction.payment == pendingPayment {
            compoundInteractionRestriction.decrease(animated: true)
            didAddMoreTime()
        }
    }
}

// MARK: - Header Bar

extension OutOfTimeViewController: RootContainment {
    var preferredHeaderBarPresentation: HeaderBarPresentation {
        .init(style: .unsecured, showsDivider: false)
    }

    var prefersHeaderBarHidden: Bool {
        false
    }
}
