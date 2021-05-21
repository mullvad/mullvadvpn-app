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

    @IBOutlet var accountTokenButton: UIButton!
    @IBOutlet var purchaseButton: InAppPurchaseButton!
    @IBOutlet var restoreButton: AppButton!
    @IBOutlet var logoutButton: AppButton!
    @IBOutlet var expiryLabel: UILabel!
    @IBOutlet var activityIndicator: SpinnerActivityIndicatorView!

    private var copyToPasteboardWork: DispatchWorkItem?

    private var pendingPayment: SKPayment?
    private let alertPresenter = AlertPresenter()
    private let logger = Logger(label: "AccountViewController")

    weak var delegate: AccountViewControllerDelegate?

    private lazy var purchaseButtonInteractionRestriction =
        UserInterfaceInteractionRestriction { [weak self] (enableUserInteraction, _) in
            // Make sure to disable the button if the product is not loaded
            self?.purchaseButton.isEnabled = enableUserInteraction &&
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

    override func viewDidLoad() {
        super.viewDidLoad()

        navigationItem.title = NSLocalizedString("Account", comment: "Navigation title")

        AppStorePaymentManager.shared.addPaymentObserver(self)
        Account.shared.addObserver(self)

        accountTokenButton.setTitle(Account.shared.formattedToken, for: .normal)

        if let expiryDate = Account.shared.expiry {
            updateAccountExpiry(expiryDate: expiryDate)
        }

        // Make sure to disable IAPs when payments are restricted
        if AppStorePaymentManager.canMakePayments {
            requestStoreProducts()
        } else {
            setPaymentsRestricted()
        }
    }

    // MARK: - Private methods

    private func updateAccountExpiry(expiryDate: Date) {
        let accountExpiry = AccountExpiry(date: expiryDate)

        if accountExpiry.isExpired {
            expiryLabel.text = NSLocalizedString("OUT OF TIME", comment: "")
            expiryLabel.textColor = .dangerColor
        } else {
            expiryLabel.text = accountExpiry.formattedDate
            expiryLabel.textColor = .white
        }
    }

    private func requestStoreProducts() {
        let inAppPurchase = AppStoreSubscription.thirtyDays

        purchaseButton.setTitle(inAppPurchase.localizedTitle, for: .normal)
        purchaseButton.isLoading = true

        purchaseButtonInteractionRestriction.increase(animated: true)

        AppStorePaymentManager.shared.requestProducts(with: [inAppPurchase]) { [weak self] (result) in
            DispatchQueue.main.async {
                guard let self = self else { return }

                switch result {
                case .success(let response):
                    if let product = response.products.first {
                        self.setProduct(product, animated: true)
                    }

                case .failure(let error):
                    self.didFailLoadingProducts(with: error)
                }

                self.purchaseButton.isLoading = false
                self.purchaseButtonInteractionRestriction.decrease(animated: true)
            }
        }
    }

    private func setProduct(_ product: SKProduct, animated: Bool) {
        self.product = product

        let localizedTitle = product.customLocalizedTitle ?? ""
        let localizedPrice = product.localizedPrice ?? ""

        let format = NSLocalizedString(
                "%1$@ (%2$@)",
                comment: "The buy button title: <TITLE> (<PRICE>). The order can be changed by swapping %1 and %2."
        )
        let title = String(format: format, localizedTitle, localizedPrice)

        purchaseButton.setTitle(title, for: .normal)
    }

    private func didFailLoadingProducts(with error: Error) {
        let title = NSLocalizedString(
            "Cannot connect to AppStore",
            comment: "The buy button title displayed when unable to load the price of subscription"
        )

        purchaseButton.setTitle(title, for: .normal)
    }

    private func setPaymentsRestricted() {
        let title = NSLocalizedString("Payments restricted", comment: "")

        purchaseButton.setTitle(title, for: .normal)
        purchaseButton.isEnabled = false
    }

    private func setEnableUserInteraction(_ enableUserInteraction: Bool, animated: Bool) {
        // Disable all buttons
        [restoreButton, logoutButton].forEach { (button) in
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
            activityIndicator.stopAnimating()
        } else {
            activityIndicator.startAnimating()
        }
    }

    private func showTimeAddedConfirmationAlert(
        with response: CreateApplePaymentResponse,
        context: CreateApplePaymentResponse.Context)
    {
        let alertController = UIAlertController(
            title: response.alertTitle(context: context),
            message: response.alertMessage(context: context),
            preferredStyle: .alert
        )
        alertController.addAction(UIAlertAction(title: NSLocalizedString("OK", comment: ""), style: .cancel))

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    private func showLogoutConfirmation(completion: @escaping (Bool) -> Void, animated: Bool) {
        let message = NSLocalizedString(
            "Are you sure you want to log out?\n\nThis will erase the account number from this device. It is not possible for us to recover it for you. Make sure you have your account number saved somewhere, to be able to log back in.",
            comment: "Alert message in log out confirmation")

        let alertController = UIAlertController(
            title: NSLocalizedString("Log out", comment: "Alert title in log out confirmation"),
            message: message,
            preferredStyle: .alert
        )

        alertController.addAction(
            UIAlertAction(
                title: NSLocalizedString("Cancel", comment: "Log out confirmation action"),
                style: .cancel,
                handler: { (alertAction) in
                    completion(false)
            })
        )

        alertController.addAction(
            UIAlertAction(
                title: NSLocalizedString("Log out", comment: "Log out confirmation action"),
                style: .destructive,
                handler: { (alertAction) in
                    completion(true)
            })
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    private func confirmLogout() {
        let message = NSLocalizedString("Logging out. Please wait...",
                                        comment: "A modal message displayed during log out")

        let alertController = UIAlertController(
            title: nil,
            message: message,
            preferredStyle: .alert)

        alertPresenter.enqueue(alertController, presentingController: self) {
            Account.shared.logout { (result) in
                DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) {
                    alertController.dismiss(animated: true) {
                        switch result {
                        case .failure(let error):
                            self.logger.error(chainedError: error, message: "Failed to log out")

                            let errorAlertController = UIAlertController(
                                title: NSLocalizedString("Failed to log out", comment: ""),
                                message: error.errorChainDescription,
                                preferredStyle: .alert
                            )
                            errorAlertController.addAction(
                                UIAlertAction(title: NSLocalizedString("OK", comment: ""), style: .cancel)
                            )
                            self.alertPresenter.enqueue(errorAlertController, presentingController: self)

                        case .success:
                            self.delegate?.accountViewControllerDidLogout(self)
                        }
                    }
                }
            }
        }
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
        DispatchQueue.main.async {
            let alertController = UIAlertController(
                title: NSLocalizedString("Cannot complete the purchase", comment: ""),
                message: error.errorChainDescription,
                preferredStyle: .alert
            )

            alertController.addAction(
                UIAlertAction(title: NSLocalizedString("OK", comment: ""), style: .cancel)
            )

            self.alertPresenter.enqueue(alertController, presentingController: self)

            if transaction.payment == self.pendingPayment {
                self.compoundInteractionRestriction.decrease(animated: true)
            }
        }
    }

    func appStorePaymentManager(_ manager: AppStorePaymentManager, transaction: SKPaymentTransaction, accountToken: String, didFinishWithResponse response: CreateApplePaymentResponse) {
        DispatchQueue.main.async {
            self.showTimeAddedConfirmationAlert(with: response, context: .purchase)

            if transaction.payment == self.pendingPayment {
                self.compoundInteractionRestriction.decrease(animated: true)
            }
        }
    }


    // MARK: - Actions

    @IBAction func doLogout() {
        showLogoutConfirmation(completion: { (confirmed) in
            if confirmed {
                self.confirmLogout()
            }
        }, animated: true)
    }

    @IBAction func copyAccountToken() {
        UIPasteboard.general.string = Account.shared.token

        accountTokenButton.setTitle(
            NSLocalizedString("COPIED TO PASTEBOARD!", comment: ""),
            for: .normal)

        let dispatchWork = DispatchWorkItem { [weak self] in
            self?.accountTokenButton.setTitle(Account.shared.formattedToken, for: .normal)
        }

        DispatchQueue.main.asyncAfter(wallDeadline: .now() + .seconds(3), execute: dispatchWork)

        self.copyToPasteboardWork?.cancel()
        self.copyToPasteboardWork = dispatchWork
    }

    @IBAction func doPurchase() {
        guard let product = product, let accountToken = Account.shared.token else { return }

        let payment = SKPayment(product: product)
        self.pendingPayment = payment

        compoundInteractionRestriction.increase(animated: true)

        AppStorePaymentManager.shared.addPayment(payment, for: accountToken)
    }

    @IBAction func restorePurchases() {
        guard let accountToken = Account.shared.token else { return }

        compoundInteractionRestriction.increase(animated: true)

        AppStorePaymentManager.shared.restorePurchases(for: accountToken) { (result) in
            DispatchQueue.main.async {
                switch result {
                case .success(let response):
                    self.showTimeAddedConfirmationAlert(with: response, context: .restoration)

                case .failure(let error):
                    let alertController = UIAlertController(
                        title: NSLocalizedString("Cannot restore purchases", comment: ""),
                        message: error.errorChainDescription,
                        preferredStyle: .alert
                    )
                    alertController.addAction(
                        UIAlertAction(title: NSLocalizedString("OK", comment: ""), style: .cancel)
                    )
                    self.alertPresenter.enqueue(alertController, presentingController: self)
                }

                self.compoundInteractionRestriction.decrease(animated: true)
            }
        }
    }

}

private extension CreateApplePaymentResponse {

    enum Context {
        case purchase
        case restoration
    }

    func alertTitle(context: Context) -> String {
        switch context {
        case .purchase:
            return NSLocalizedString("Thanks for your purchase", comment: "")
        case .restoration:
            return NSLocalizedString("Restore purchases", comment: "")
        }
    }

    func alertMessage(context: Context) -> String {
        switch context {
        case .purchase:
            return String(
                format: NSLocalizedString("%@ have been added to your account", comment: ""),
                formattedTimeAdded ?? ""
            )
        case .restoration:
            switch self {
            case .noTimeAdded:
                return NSLocalizedString(
                    "Your previous purchases have already been added to this account.",
                    comment: "")
            case .timeAdded:
                return String(
                    format: NSLocalizedString("%@ have been added to your account", comment: ""),
                    self.formattedTimeAdded ?? ""
                )
            }
        }
    }
}
