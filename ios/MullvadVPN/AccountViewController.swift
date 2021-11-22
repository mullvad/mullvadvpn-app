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

class AccountViewController: UIViewController, AppStorePaymentObserver, AccountObserver {

    private let contentView: AccountContentView = {
        let contentView = AccountContentView()
        contentView.translatesAutoresizingMaskIntoConstraints = false
        return contentView
    }()

    private var copyToPasteboardCancellationToken: PromiseCancellationToken?

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
            comment: "Navigation title"
        )

        contentView.accountTokenRowView.value = Account.shared.formattedToken
        contentView.accountTokenRowView.actionHandler = { [weak self] in
            self?.copyAccountToken()
        }

        contentView.restorePurchasesButton.addTarget(self, action: #selector(restorePurchases), for: .touchUpInside)
        contentView.purchaseButton.addTarget(self, action: #selector(doPurchase), for: .touchUpInside)
        contentView.logoutButton.addTarget(self, action: #selector(doLogout), for: .touchUpInside)

        AppStorePaymentManager.shared.addPaymentObserver(self)
        Account.shared.addObserver(self)

        updateAccountExpiry(expiryDate: Account.shared.expiry)

        // Make sure to disable IAPs when payments are restricted
        if AppStorePaymentManager.canMakePayments {
            requestStoreProducts()
        } else {
            setPaymentsRestricted()
        }
    }

    // MARK: - Private methods

    private func updateAccountExpiry(expiryDate: Date?) {
        contentView.accountExpiryRowView.value = expiryDate
    }

    private func requestStoreProducts() {
        let inAppPurchase = AppStoreSubscription.thirtyDays

        contentView.purchaseButton.setTitle(inAppPurchase.localizedTitle, for: .normal)
        contentView.purchaseButton.isLoading = true

        purchaseButtonInteractionRestriction.increase(animated: true)

        AppStorePaymentManager.shared.requestProducts(with: [inAppPurchase])
            .receive(on: .main)
            .observe { [weak self] completion in
                guard let self = self else { return }

                if let result = completion.unwrappedValue {
                    switch result {
                    case .success(let response):
                        if let product = response.products.first {
                            self.setProduct(product, animated: true)
                        }

                    case .failure(let error):
                        self.didFailLoadingProducts(with: error)
                    }
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
            comment: "Purchase button title: <TITLE> (<PRICE>). The order can be changed by swapping %1 and %2."
        )
        let title = String(format: format, localizedTitle, localizedPrice)

        contentView.purchaseButton.setTitle(title, for: .normal)
    }

    private func didFailLoadingProducts(with error: Error) {
        let title = NSLocalizedString(
            "PURCHASE_BUTTON_CANNOT_CONNECT_TO_APPSTORE_LABEL",
            tableName: "Account",
            value: "Cannot connect to AppStore",
            comment: "Purchase button title displayed when unable to load the price of in-app purchase."
        )

        contentView.purchaseButton.setTitle(title, for: .normal)
    }

    private func setPaymentsRestricted() {
        let title = NSLocalizedString(
            "PURCHASE_BUTTON_PAYMENTS_RESTRICTED_LABEL",
            tableName: "Account",
            value: "Payments restricted",
            comment: "Purchase button title displayed when payments are restriced on device."
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

    private func showLogoutConfirmation(completion: @escaping (Bool) -> Void, animated: Bool) {
        let alertController = UIAlertController(
            title: NSLocalizedString(
                "LOGOUT_CONFIRMATION_ALERT_TITLE",
                tableName: "Account",
                comment: "Title for logout dialog"
            ),
            message: NSLocalizedString(
                "LOGOUT_CONFIRMATION_ALERT_MESSAGE",
                tableName: "Account",
                comment: "Message for logout dialog"
            ),
            preferredStyle: .alert
        )

        alertController.addAction(
            UIAlertAction(
                title: NSLocalizedString(
                    "LOGOUT_CONFIRMATION_ALERT_CANCEL_ACTION",
                    tableName: "Account",
                    value: "Cancel",
                    comment: "Title for cancel button in logout dialog"
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
                    comment: "Title for confirmation button in logout dialog"
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
            comment: "Modal message displayed during logout"
        )

        let alertController = UIAlertController(
            title: nil,
            message: message,
            preferredStyle: .alert
        )

        alertPresenter.enqueue(alertController, presentingController: self) {
            Account.shared.logout()
                .receive(on: .main, after: .seconds(1), timerType: .deadline)
                .then { result in
                    return Promise { resolver in
                        alertController.dismiss(animated: true) {
                            resolver.resolve(value: result)
                        }
                    }
                }
                .onSuccess { _ in
                    self.delegate?.accountViewControllerDidLogout(self)
                }
                .onFailure { error in
                    self.logger.error(chainedError: error, message: "Failed to log out")

                    self.showLogoutFailure(error)
                }
                .observe { _ in }
        }
    }

    private func showLogoutFailure(_ error: Account.Error) {
        let errorAlertController = UIAlertController(
            title: NSLocalizedString(
                "LOGOUT_FAILURE_ALERT_TITLE",
                tableName: "Account",
                value: "Failed to log out",
                comment: "Title for logout failure alert"
            ),
            message: error.errorChainDescription,
            preferredStyle: .alert
        )
        errorAlertController.addAction(
            UIAlertAction(title: NSLocalizedString(
                "LOGOUT_FAILURE_ALERT_OK_ACTION",
                tableName: "Account",
                value: "OK",
                comment: "Message for logout failure alert"
            ), style: .cancel)
        )
        alertPresenter.enqueue(errorAlertController, presentingController: self)
    }

    // MARK: - AccountObserver

    func account(_ account: Account, didUpdateExpiry expiry: Date) {
        updateAccountExpiry(expiryDate: expiry)
    }

    func account(_ account: Account, didLoginWithToken token: String, expiry: Date) {
        // no-op
    }

    func accountDidLogout(_ account: Account) {
        // no-op
    }

    // MARK: - AppStorePaymentObserver

    func appStorePaymentManager(_ manager: AppStorePaymentManager, transaction: SKPaymentTransaction, accountToken: String?, didFailWithError error: AppStorePaymentManager.Error) {
        let alertController = UIAlertController(
            title: NSLocalizedString(
                "CANNOT_COMPLETE_PURCHASE_ALERT_TITLE",
                tableName: "Account",
                value: "Cannot complete the purchase",
                comment: "Title for purchase failure dialog"
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
                    comment: "Title for OK button in purchase failure dialog"
                ), style: .cancel)
        )

        alertPresenter.enqueue(alertController, presentingController: self)

        if transaction.payment == pendingPayment {
            compoundInteractionRestriction.decrease(animated: true)
        }
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager, transaction: SKPaymentTransaction, accountToken: String, didFinishWithResponse response: REST.CreateApplePaymentResponse) {
        showTimeAddedConfirmationAlert(with: response, context: .purchase)

        if transaction.payment == self.pendingPayment {
            compoundInteractionRestriction.decrease(animated: true)
        }
    }


    // MARK: - Actions

    @objc private func doLogout() {
        showLogoutConfirmation(completion: { (confirmed) in
            if confirmed {
                self.confirmLogout()
            }
        }, animated: true)
    }

    private func copyAccountToken() {
        UIPasteboard.general.string = Account.shared.token

        contentView.accountTokenRowView.value = NSLocalizedString(
            "COPIED_TO_PASTEBOARD_LABEL",
            tableName: "Account",
            comment: "Message, temporarily displayed in place account token, after copying the account token to pasteboard on tap."
        )

        Promise.deferred { Account.shared.formattedToken }
            .delay(by: .seconds(3), timerType: .walltime, queue: .main)
            .storeCancellationToken(in: &copyToPasteboardCancellationToken)
            .observe { [weak self] completion in
                guard let formattedToken = completion.unwrappedValue else { return }

                self?.contentView.accountTokenRowView.value = formattedToken
            }
    }

    @objc private func doPurchase() {
        guard let product = product, let accountToken = Account.shared.token else { return }

        let payment = SKPayment(product: product)

        pendingPayment = payment
        compoundInteractionRestriction.increase(animated: true)

        AppStorePaymentManager.shared.addPayment(payment, for: accountToken)
    }

    @objc private func restorePurchases() {
        guard let accountToken = Account.shared.token else { return }

        compoundInteractionRestriction.increase(animated: true)

        AppStorePaymentManager.shared.restorePurchases(for: accountToken)
            .receive(on: .main)
            .onSuccess { response in
                self.showTimeAddedConfirmationAlert(with: response, context: .restoration)
            }
            .onFailure { error in
                let alertController = UIAlertController(
                    title: NSLocalizedString(
                        "RESTORE_PURCHASES_FAILURE_ALERT_TITLE",
                        tableName: "Account",
                        value: "Cannot restore purchases",
                        comment: "Title for failure dialog when restoring purchases"
                    ),
                    message: error.errorChainDescription,
                    preferredStyle: .alert
                )
                alertController.addAction(
                    UIAlertAction(title: NSLocalizedString(
                        "RESTORE_PURCHASES_FAILURE_ALERT_OK_ACTION",
                        tableName: "Account",
                        value: "OK",
                        comment: "Title for 'OK' button in failure dialog when restoring purchases"
                    ), style: .cancel)
                )
                self.alertPresenter.enqueue(alertController, presentingController: self)
            }
            .observe { _ in
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
                comment: "Title for purchase completion dialog"
            )
        case .restoration:
            return NSLocalizedString(
                "RESTORE_PURCHASES_ALERT_TITLE",
                tableName: "Account", value: "Restore purchases",
                comment: "Title for purchase restoration dialog"
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
                    comment: "Message displayed upon successful purchase and containing the time duration credited to user account. Use %@ placeholder to position the localized text with duration added (i.e '30 days')"
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
                    comment: "Message displayed when no time credited to user account during purchase restoration; communicates that user account has already been credited with all outstanding purchased time duration."
                )
            case .timeAdded:
                return String(
                    format: NSLocalizedString(
                        "RESTORE_PURCHASES_ALERT_TIME_ADDED_MESSAGE",
                        tableName: "Account",
                        value: "%@ have been added to your account",
                        comment: "Message displayed upon successful restoration of existing purchases, containing the time duration credited to user account. Use %@ placeholder to position the localized text with duration added (i.e '30 days')"
                    ),
                    formattedTimeAdded ?? ""
                )
            }
        }
    }
}
