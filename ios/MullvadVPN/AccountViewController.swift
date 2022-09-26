//
//  AccountViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Logging
import StoreKit
import UIKit

protocol AccountViewControllerDelegate: AnyObject {
    func accountViewControllerDidLogout(_ controller: AccountViewController)
}

class AccountViewController: UIViewController, AppStorePaymentObserver, TunnelObserver {
    private let alertPresenter = AlertPresenter()

    private let contentView: AccountContentView = {
        let contentView = AccountContentView()
        contentView.translatesAutoresizingMaskIntoConstraints = false
        return contentView
    }()

    private var productState: ProductState = .none
    private var paymentState: PaymentState = .none

    weak var delegate: AccountViewControllerDelegate?

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

        AppStorePaymentManager.shared.addPaymentObserver(self)
        TunnelManager.shared.addObserver(self)

        updateView(from: TunnelManager.shared.deviceState)
        applyViewState(animated: false)

        if AppStorePaymentManager.canMakePayments {
            requestStoreProducts()
        } else {
            setProductState(.cannotMakePurchases, animated: false)
        }
    }

    // MARK: - Private methods

    private func requestStoreProducts() {
        let productKind = AppStoreSubscription.thirtyDays

        setProductState(.fetching(productKind), animated: true)

        _ = AppStorePaymentManager.shared
            .requestProducts(with: [productKind]) { [weak self] completion in
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

    private func didProcessPayment(_ payment: SKPayment) {
        guard case let .makingPayment(pendingPayment) = paymentState,
              pendingPayment == payment else { return }

        setPaymentState(.none, animated: true)
    }

    private func showPaymentErrorAlert(error: AppStorePaymentManager.Error) {
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

    private func showRestorePurchasesErrorAlert(error: AppStorePaymentManager.Error) {
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
            TunnelManager.shared.unsetAccount {
                DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) {
                    alertController.dismiss(animated: true) {
                        self.delegate?.accountViewControllerDidLogout(self)
                    }
                }
            }
        }
    }

    // MARK: - TunnelObserver

    func tunnelManagerDidLoadConfiguration(_ manager: TunnelManager) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelStatus tunnelStatus: TunnelStatus) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: Error) {
        // no-op
    }

    func tunnelManager(
        _ manager: TunnelManager,
        didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2
    ) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateDeviceState deviceState: DeviceState) {
        updateView(from: deviceState)
    }

    // MARK: - AppStorePaymentObserver

    func appStorePaymentManager(
        _ manager: AppStorePaymentManager,
        transaction: SKPaymentTransaction?,
        payment: SKPayment,
        accountToken: String?,
        didFailWithError error: AppStorePaymentManager.Error
    ) {
        switch error {
        case .storePayment(SKError.paymentCancelled):
            break

        default:
            showPaymentErrorAlert(error: error)
        }

        didProcessPayment(payment)
    }

    func appStorePaymentManager(
        _ manager: AppStorePaymentManager,
        transaction: SKPaymentTransaction,
        accountToken: String,
        didFinishWithResponse response: REST.CreateApplePaymentResponse
    ) {
        showTimeAddedConfirmationAlert(with: response, context: .purchase)

        didProcessPayment(transaction.payment)
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
        guard let accountData = TunnelManager.shared.deviceState.accountData else {
            return
        }

        UIPasteboard.general.string = accountData.number
    }

    @objc private func doPurchase() {
        guard case let .received(product) = productState,
              let accountData = TunnelManager.shared.deviceState.accountData
        else {
            return
        }

        let payment = SKPayment(product: product)
        AppStorePaymentManager.shared.addPayment(payment, for: accountData.number)

        setPaymentState(.makingPayment(payment), animated: true)
    }

    @objc private func restorePurchases() {
        guard let accountData = TunnelManager.shared.deviceState.accountData else {
            return
        }

        setPaymentState(.restoringPurchases, animated: true)

        _ = AppStorePaymentManager.shared.restorePurchases(for: accountData.number) { completion in
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

private extension AccountViewController {
    enum PaymentState: Equatable {
        case none
        case makingPayment(SKPayment)
        case restoringPurchases

        var allowsViewInteraction: Bool {
            switch self {
            case .none:
                return true
            case .restoringPurchases, .makingPayment:
                return false
            }
        }
    }

    enum ProductState {
        case none
        case fetching(AppStoreSubscription)
        case received(SKProduct)
        case failed
        case cannotMakePurchases

        var isFetching: Bool {
            if case .fetching = self {
                return true
            }
            return false
        }

        var isReceived: Bool {
            if case .received = self {
                return true
            }
            return false
        }

        var purchaseButtonTitle: String? {
            switch self {
            case .none:
                return nil

            case let .fetching(subscription):
                return subscription.localizedTitle

            case let .received(product):
                let localizedTitle = product.customLocalizedTitle ?? ""
                let localizedPrice = product.localizedPrice ?? ""

                let format = NSLocalizedString(
                    "PURCHASE_BUTTON_TITLE_FORMAT",
                    tableName: "Account",
                    value: "%1$@ (%2$@)",
                    comment: ""
                )
                return String(format: format, localizedTitle, localizedPrice)

            case .failed:
                return NSLocalizedString(
                    "PURCHASE_BUTTON_CANNOT_CONNECT_TO_APPSTORE_LABEL",
                    tableName: "Account",
                    value: "Cannot connect to AppStore",
                    comment: ""
                )

            case .cannotMakePurchases:
                return NSLocalizedString(
                    "PURCHASE_BUTTON_PAYMENTS_RESTRICTED_LABEL",
                    tableName: "Account",
                    value: "Payments restricted",
                    comment: ""
                )
            }
        }
    }
}
