//
//  AccountViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import StoreKit
import UIKit
import Logging

protocol AccountViewControllerDelegate: AnyObject {
    func accountViewControllerDidLogout(_ controller: AccountViewController)
}

class AccountViewController: UIViewController, AppStorePaymentObserver, TunnelObserver {
    private let contentView: AccountContentView = {
        let contentView = AccountContentView()
        contentView.translatesAutoresizingMaskIntoConstraints = false
        return contentView
    }()

    private var copyToPasteboardWork: DispatchWorkItem?

    private var pendingPayment: SKPayment?
    private let alertPresenter = AlertPresenter()
    private let logger = Logger(label: "AccountViewController")

    weak var delegate: AccountViewControllerDelegate?

    private lazy var purchaseButtonInteractionRestriction =
        UserInterfaceInteractionRestriction { [weak self] (enableUserInteraction, _) in
            // Make sure to disable the button if the product is not loaded
            self?.contentView.purchaseButton.isEnabled = enableUserInteraction &&
                self?.product != nil &&
                AppStorePaymentManager.canMakePayments
    }

    private lazy var viewControllerInteractionRestriction =
        UserInterfaceInteractionRestriction { [weak self] (enableUserInteraction, animated) in
            self?.setEnableUserInteraction(enableUserInteraction, animated: true)
    }

    private lazy var compoundInteractionRestriction =
        CompoundUserInterfaceInteractionRestriction(restrictions: [
            purchaseButtonInteractionRestriction, viewControllerInteractionRestriction])

    private var product: SKProduct?

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
            contentView.bottomAnchor.constraint(greaterThanOrEqualTo: scrollView.safeAreaLayoutGuide.bottomAnchor),
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

        contentView.accountTokenRowView.accountNumber = TunnelManager.shared.accountNumber
        contentView.accountTokenRowView.copyAccountNumber = { [weak self] in
            self?.copyAccountToken()
        }

        contentView.restorePurchasesButton.addTarget(self, action: #selector(restorePurchases), for: .touchUpInside)
        contentView.purchaseButton.addTarget(self, action: #selector(doPurchase), for: .touchUpInside)
        contentView.logoutButton.addTarget(self, action: #selector(doLogout), for: .touchUpInside)

        AppStorePaymentManager.shared.addPaymentObserver(self)
        TunnelManager.shared.addObserver(self)

        updateAccountExpiry(expiryDate: TunnelManager.shared.accountExpiry)
        updateDeviceName(TunnelManager.shared.device?.name)

        // Make sure to disable IAPs when payments are restricted
        if AppStorePaymentManager.canMakePayments {
            requestStoreProducts()
        } else {
            setPaymentsRestricted()
        }
    }

    // MARK: - Private methods

    private func updateDeviceName(_ deviceName: String?) {
        contentView.accountDeviceRow.deviceName = deviceName
    }

    private func updateAccountExpiry(expiryDate: Date?) {
        contentView.accountExpiryRowView.value = expiryDate
    }

    private func requestStoreProducts() {
        let inAppPurchase = AppStoreSubscription.thirtyDays

        contentView.purchaseButton.setTitle(inAppPurchase.localizedTitle, for: .normal)
        contentView.purchaseButton.isLoading = true

        purchaseButtonInteractionRestriction.increase(animated: true)

        _ = AppStorePaymentManager.shared.requestProducts(with: [inAppPurchase]) { [weak self] completion in
            guard let self = self else { return }

            switch completion {
            case .success(let response):
                if let product = response.products.first {
                    self.setProduct(product, animated: true)
                }

            case .failure(let error):
                self.didFailLoadingProducts(with: error)

            case .cancelled:
                break
            }

            self.contentView.purchaseButton.isLoading = false
            self.purchaseButtonInteractionRestriction.decrease(animated: true)
        }
    }

    private func setProduct(_ product: SKProduct, animated: Bool) {
        self.product = product

        let localizedTitle = product.customLocalizedTitle ?? ""
        let localizedPrice = product.localizedPrice ?? ""

        let format = NSLocalizedString(
            "PURCHASE_BUTTON_TITLE_FORMAT",
            tableName: "Account",
            value: "%1$@ (%2$@)",
            comment: ""
        )
        let title = String(format: format, localizedTitle, localizedPrice)

        contentView.purchaseButton.setTitle(title, for: .normal)
    }

    private func didFailLoadingProducts(with error: Error) {
        let title = NSLocalizedString(
            "PURCHASE_BUTTON_CANNOT_CONNECT_TO_APPSTORE_LABEL",
            tableName: "Account",
            value: "Cannot connect to AppStore",
            comment: ""
        )

        contentView.purchaseButton.setTitle(title, for: .normal)
    }

    private func setPaymentsRestricted() {
        let title = NSLocalizedString(
            "PURCHASE_BUTTON_PAYMENTS_RESTRICTED_LABEL",
            tableName: "Account",
            value: "Payments restricted",
            comment: ""
        )

        contentView.purchaseButton.setTitle(title, for: .normal)
        contentView.purchaseButton.isEnabled = false
    }

    private func setEnableUserInteraction(_ enableUserInteraction: Bool, animated: Bool) {
        // Disable all buttons
        [contentView.restorePurchasesButton, contentView.logoutButton].forEach { (button) in
            button?.isEnabled = enableUserInteraction
        }

        // Disable any interaction within the view
        view.isUserInteractionEnabled = enableUserInteraction

        // Prevent view controller from being swiped away by user
        if #available(iOS 13.0, *) {
            isModalInPresentation = !enableUserInteraction
        } else {
            // Fallback on earlier versions
        }

        // Hide back button in navigation bar
        navigationItem.setHidesBackButton(!enableUserInteraction, animated: animated)

        // Show/hide the spinner next to "Paid until"
        if enableUserInteraction {
            contentView.accountExpiryRowView.activityIndicator.stopAnimating()
        } else {
            contentView.accountExpiryRowView.activityIndicator.startAnimating()
        }
    }

    private func showTimeAddedConfirmationAlert(
        with response: REST.CreateApplePaymentResponse,
        context: REST.CreateApplePaymentResponse.Context)
    {
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
                handler: { (alertAction) in
                    completion(false)
            })
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
                handler: { (alertAction) in
                    completion(true)
            })
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

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelState tunnelState: TunnelState) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: TunnelManager.Error) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2?) {
        guard let tunnelSettings = tunnelSettings else {
            return
        }

        updateDeviceName(tunnelSettings.device.name)
        updateAccountExpiry(expiryDate: tunnelSettings.account.expiry)
    }

    // MARK: - AppStorePaymentObserver

    func appStorePaymentManager(_ manager: AppStorePaymentManager, transaction: SKPaymentTransaction?, payment: SKPayment, accountToken: String?, didFailWithError error: AppStorePaymentManager.Error) {
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
                ), style: .cancel)
        )

        alertPresenter.enqueue(alertController, presentingController: self)

        if payment == pendingPayment {
            compoundInteractionRestriction.decrease(animated: true)
        }
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager, transaction: SKPaymentTransaction, accountToken: String, didFinishWithResponse response: REST.CreateApplePaymentResponse) {
        showTimeAddedConfirmationAlert(with: response, context: .purchase)

        if transaction.payment == pendingPayment {
            compoundInteractionRestriction.decrease(animated: true)
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
        UIPasteboard.general.string = TunnelManager.shared.accountNumber
    }

    @objc private func doPurchase() {
        guard let product = product, let accountNumber = TunnelManager.shared.accountNumber else { return }

        let payment = SKPayment(product: product)

        pendingPayment = payment
        compoundInteractionRestriction.increase(animated: true)

        AppStorePaymentManager.shared.addPayment(payment, for: accountNumber)
    }

    @objc private func restorePurchases() {
        guard let accountNumber = TunnelManager.shared.accountNumber  else { return }

        compoundInteractionRestriction.increase(animated: true)

        _ = AppStorePaymentManager.shared.restorePurchases(for: accountNumber) { completion in
            switch completion {
            case .success(let response):
                self.showTimeAddedConfirmationAlert(with: response, context: .restoration)

            case .failure(let error):
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
                self.alertPresenter.enqueue(alertController, presentingController: self)

            case .cancelled:
                break
            }

            self.compoundInteractionRestriction.decrease(animated: true)
        }
    }

}

private extension REST.CreateApplePaymentResponse {

    enum Context {
        case purchase
        case restoration
    }

    func alertTitle(context: Context) -> String {
        switch context {
        case .purchase:
            return NSLocalizedString(
                "TIME_ADDED_ALERT_SUCCESS_TITLE",
                tableName: "Account",
                value: "Thanks for your purchase",
                comment: ""
            )
        case .restoration:
            return NSLocalizedString(
                "RESTORE_PURCHASES_ALERT_TITLE",
                tableName: "Account",
                value: "Restore purchases",
                comment: ""
            )
        }
    }

    func alertMessage(context: Context) -> String {
        switch context {
        case .purchase:
            return String(
                format: NSLocalizedString(
                    "TIME_ADDED_ALERT_SUCCESS_MESSAGE",
                    tableName: "Account",
                    value: "%@ have been added to your account",
                    comment: ""
                ),
                formattedTimeAdded ?? ""
            )
        case .restoration:
            switch self {
            case .noTimeAdded:
                return NSLocalizedString(
                    "RESTORE_PURCHASES_ALERT_NO_TIME_ADDED_MESSAGE",
                    tableName: "Account",
                    value: "Your previous purchases have already been added to this account.",
                    comment: ""
                )
            case .timeAdded:
                return String(
                    format: NSLocalizedString(
                        "RESTORE_PURCHASES_ALERT_TIME_ADDED_MESSAGE",
                        tableName: "Account",
                        value: "%@ have been added to your account",
                        comment: ""
                    ),
                    formattedTimeAdded ?? ""
                )
            }
        }
    }
}
