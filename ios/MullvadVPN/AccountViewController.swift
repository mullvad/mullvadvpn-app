//
//  AccountViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import StoreKit
import UIKit

protocol AccountViewControllerDelegate: AnyObject {
    func accountViewControllerDidLogout(_ controller: AccountViewController)
}

class AccountViewController: UIViewController {
    private let interactor: AccountInteractor
    private let alertPresenter = AlertPresenter()

    private let contentView: AccountContentView = {
        let contentView = AccountContentView()
        contentView.translatesAutoresizingMaskIntoConstraints = false
        return contentView
    }()

    private var productState: ProductState = .none
    private var paymentState: PaymentState = .none

    weak var delegate: AccountViewControllerDelegate?

    init(interactor: AccountInteractor) {
        self.interactor = interactor

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - View lifecycle

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor

        let scrollView = UIScrollView()
        scrollView.translatesAutoresizingMaskIntoConstraints = false
        scrollView.addSubview(contentView)
        view.addSubview(scrollView)

        NSLayoutConstraint.activate([
            scrollView.topAnchor.constraint(equalTo: view.topAnchor),
            scrollView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            scrollView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            scrollView.bottomAnchor.constraint(equalTo: view.bottomAnchor),

            contentView.topAnchor.constraint(equalTo: scrollView.topAnchor),
            contentView.bottomAnchor
                .constraint(greaterThanOrEqualTo: scrollView.safeAreaLayoutGuide.bottomAnchor),
            contentView.leadingAnchor.constraint(equalTo: scrollView.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: scrollView.trailingAnchor),
            contentView.widthAnchor.constraint(equalTo: scrollView.widthAnchor),
        ])

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "Account",
            value: "Account",
            comment: ""
        )

        contentView.accountTokenRowView.copyAccountNumber = { [weak self] in
            self?.copyAccountToken()
        }

        contentView.restorePurchasesButton.addTarget(
            self,
            action: #selector(restorePurchases),
            for: .touchUpInside
        )
        contentView.purchaseButton.addTarget(
            self,
            action: #selector(doPurchase),
            for: .touchUpInside
        )
        contentView.logoutButton.addTarget(self, action: #selector(doLogout), for: .touchUpInside)

        interactor.didReceiveDeviceState = { [weak self] newDeviceState in
            self?.updateView(from: newDeviceState)
        }

        interactor.didReceivePaymentEvent = { [weak self] event in
            self?.didReceivePaymentEvent(event)
        }

        updateView(from: interactor.deviceState)
        applyViewState(animated: false)

        if StorePaymentManager.canMakePayments {
            requestStoreProducts()
        } else {
            setProductState(.cannotMakePurchases, animated: false)
        }
    }

    // MARK: - Private

    private func requestStoreProducts() {
        let productKind = StoreSubscription.thirtyDays

        setProductState(.fetching(productKind), animated: true)

        _ = interactor.requestProducts(with: [productKind]) { [weak self] completion in
            let productState: ProductState = completion.value?.products.first
                .map { .received($0) } ?? .failed

            self?.setProductState(productState, animated: true)
        }
    }

    private func setPaymentState(_ newState: PaymentState, animated: Bool) {
        paymentState = newState

        applyViewState(animated: animated)
    }

    private func setProductState(_ newState: ProductState, animated: Bool) {
        productState = newState

        applyViewState(animated: animated)
    }

    private func updateView(from deviceState: DeviceState?) {
        guard case let .loggedIn(accountData, deviceData) = deviceState else {
            return
        }

        contentView.accountDeviceRow.deviceName = deviceData.name
        contentView.accountTokenRowView.accountNumber = accountData.number
        contentView.accountExpiryRowView.value = accountData.expiry
    }

    private func applyViewState(animated: Bool) {
        let isInteractionEnabled = paymentState.allowsViewInteraction
        let purchaseButton = contentView.purchaseButton
        let activityIndicator = contentView.accountExpiryRowView.activityIndicator

        if productState.isFetching || paymentState != .none {
            activityIndicator.startAnimating()
        } else {
            activityIndicator.stopAnimating()
        }

        purchaseButton.setTitle(productState.purchaseButtonTitle, for: .normal)
        contentView.purchaseButton.isLoading = productState.isFetching

        purchaseButton.isEnabled = productState.isReceived && isInteractionEnabled
        contentView.restorePurchasesButton.isEnabled = isInteractionEnabled
        contentView.logoutButton.isEnabled = isInteractionEnabled

        view.isUserInteractionEnabled = isInteractionEnabled
        isModalInPresentation = !isInteractionEnabled

        navigationItem.setHidesBackButton(!isInteractionEnabled, animated: animated)
    }

    private func didReceivePaymentEvent(_ event: StorePaymentEvent) {
        guard case let .makingPayment(payment) = paymentState,
              payment == event.payment else { return }

        switch event {
        case let .finished(completion):
            showTimeAddedConfirmationAlert(with: completion.serverResponse, context: .purchase)

        case let .failure(paymentFailure):
            switch paymentFailure.error {
            case .storePayment(SKError.paymentCancelled):
                break

            default:
                showPaymentErrorAlert(error: paymentFailure.error)
            }
        }

        setPaymentState(.none, animated: true)
    }

    private func showPaymentErrorAlert(error: StorePaymentManagerError) {
        let alertController = UIAlertController(
            title: NSLocalizedString(
                "CANNOT_COMPLETE_PURCHASE_ALERT_TITLE",
                tableName: "Account",
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
                    tableName: "Account",
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
                tableName: "Account",
                value: "Cannot restore purchases",
                comment: ""
            ),
            message: error.errorChainDescription,
            preferredStyle: .alert
        )
        alertController.addAction(
            UIAlertAction(title: NSLocalizedString(
                "RESTORE_PURCHASES_FAILURE_ALERT_OK_ACTION",
                tableName: "Account",
                value: "OK",
                comment: ""
            ), style: .cancel)
        )
        alertPresenter.enqueue(alertController, presentingController: self)
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
                    tableName: "Account",
                    value: "OK",
                    comment: ""
                ),
                style: .cancel
            )
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    private func showLogoutConfirmation(animated: Bool, completion: @escaping (Bool) -> Void) {
        let alertController = UIAlertController(
            title: NSLocalizedString(
                "LOGOUT_CONFIRMATION_ALERT_TITLE",
                tableName: "Account",
                value: "Log out",
                comment: ""
            ),
            message: NSLocalizedString(
                "LOGOUT_CONFIRMATION_ALERT_MESSAGE",
                tableName: "Account",
                value: "Are you sure you want to log out?\n\nThis will erase the account number from this device. It is not possible for us to recover it for you. Make sure you have your account number saved somewhere, to be able to log back in.",
                comment: ""
            ),
            preferredStyle: .alert
        )

        alertController.addAction(
            UIAlertAction(
                title: NSLocalizedString(
                    "LOGOUT_CONFIRMATION_ALERT_CANCEL_ACTION",
                    tableName: "Account",
                    value: "Cancel",
                    comment: ""
                ),
                style: .cancel,
                handler: { alertAction in
                    completion(false)
                }
            )
        )

        alertController.addAction(
            UIAlertAction(
                title: NSLocalizedString(
                    "LOGOUT_CONFIRMATION_ALERT_YES_ACTION",
                    tableName: "Account",
                    value: "Log out",
                    comment: ""
                ),
                style: .destructive,
                handler: { alertAction in
                    completion(true)
                }
            )
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    private func confirmLogout() {
        let message = NSLocalizedString(
            "LOGGING_OUT_ALERT_TITLE",
            tableName: "Account",
            value: "Logging out. Please wait...",
            comment: ""
        )

        let alertController = UIAlertController(
            title: nil,
            message: message,
            preferredStyle: .alert
        )

        alertPresenter.enqueue(alertController, presentingController: self) {
            self.interactor.logout {
                DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) {
                    alertController.dismiss(animated: true) {
                        self.delegate?.accountViewControllerDidLogout(self)
                    }
                }
            }
        }
    }

    // MARK: - Actions

    @objc private func doLogout() {
        showLogoutConfirmation(animated: true) { confirmed in
            if confirmed {
                self.confirmLogout()
            }
        }
    }

    private func copyAccountToken() {
        guard let accountData = interactor.deviceState.accountData else {
            return
        }

        UIPasteboard.general.string = accountData.number
    }

    @objc private func doPurchase() {
        guard case let .received(product) = productState,
              let accountData = interactor.deviceState.accountData
        else {
            return
        }

        let payment = SKPayment(product: product)
        interactor.addPayment(payment, for: accountData.number)

        setPaymentState(.makingPayment(payment), animated: true)
    }

    @objc private func restorePurchases() {
        guard let accountData = interactor.deviceState.accountData else {
            return
        }

        setPaymentState(.restoringPurchases, animated: true)

        _ = interactor.restorePurchases(for: accountData.number) { [weak self] completion in
            guard let self = self else { return }

            switch completion {
            case let .success(response):
                self.showTimeAddedConfirmationAlert(with: response, context: .restoration)

            case let .failure(error):
                self.showRestorePurchasesErrorAlert(error: error)

            case .cancelled:
                break
            }

            self.setPaymentState(.none, animated: true)
        }
    }
}
