//
//  AccountViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Combine
import StoreKit
import UIKit
import os

class AccountViewController: UIViewController {

    @IBOutlet var accountTokenButton: UIButton!
    @IBOutlet var purchaseButton: InAppPurchaseButton!
    @IBOutlet var restoreButton: AppButton!
    @IBOutlet var logoutButton: AppButton!
    @IBOutlet var expiryLabel: UILabel!
    @IBOutlet var activityIndicator: SpinnerActivityIndicatorView!

    private var accountExpirySubscriber: AnyCancellable?
    private var logoutSubscriber: AnyCancellable?
    private var copyToPasteboardSubscriber: AnyCancellable?
    private var requestProductsSubscriber: AnyCancellable?
    private var purchaseSubscriber: AnyCancellable?
    private var restorePurchasesSubscriber: AnyCancellable?

    private lazy var purchaseButtonInteractionRestriction =
        UserInterfaceInteractionRestriction(scheduler: DispatchQueue.main) {
            [weak self] (enableUserInteraction, _) in
            // Make sure to disable the button if the product is not loaded
            self?.purchaseButton.isEnabled = enableUserInteraction &&
                self?.product != nil &&
                AppStorePaymentManager.canMakePayments
    }

    private lazy var viewControllerInteractionRestriction =
        UserInterfaceInteractionRestriction(scheduler: DispatchQueue.main) {
            [weak self] (enableUserInteraction, animated) in
            self?.setEnableUserInteraction(enableUserInteraction, animated: true)
    }

    private lazy var compoundInteractionRestriction =
        CompoundUserInterfaceInteractionRestriction(restrictions: [
            purchaseButtonInteractionRestriction, viewControllerInteractionRestriction])

    private var product: SKProduct?

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        accountExpirySubscriber = NotificationCenter.default
            .publisher(for: Account.didUpdateAccountExpiryNotification, object: Account.shared)
            .receive(on: DispatchQueue.main)
            .sink { [weak self] (notification) in
                guard let newExpiryDate = notification
                    .userInfo?[Account.newAccountExpiryUserInfoKey] as? Date else { return }

                self?.updateAccountExpiry(expiryDate: newExpiryDate)
        }

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

        requestProductsSubscriber = AppStorePaymentManager.shared.requestProducts(with: [inAppPurchase])
            .retry(10)
            .receive(on: DispatchQueue.main)
            .restrictUserInterfaceInteraction(with: self.purchaseButtonInteractionRestriction, animated: true)
            .sink(receiveCompletion: { [weak self] (completion) in
                if case .failure(let error) = completion {
                    self?.didFailLoadingProducts(with: error)
                }

                self?.purchaseButton.isLoading = false
                }, receiveValue: { [weak self] (response) in
                    if let product = response.products.first {
                        self?.setProduct(product, animated: true)
                    }
            })
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
        isModalInPresentation = !enableUserInteraction

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
        with response: SendAppStoreReceiptResponse,
        context: SendAppStoreReceiptResponse.Context)
    {
        let alertController = UIAlertController(
            title: response.alertTitle(context: context),
            message: response.alertMessage(context: context),
            preferredStyle: .alert
        )
        alertController.addAction(UIAlertAction(title: NSLocalizedString("OK", comment: ""), style: .cancel))

        present(alertController, animated: true)
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

        present(alertController, animated: animated)
    }

    private func confirmLogout() {
        let message = NSLocalizedString("Logging out. Please wait...",
                                        comment: "A modal message displayed during log out")

        let alertController = UIAlertController(
            title: nil,
            message: message,
            preferredStyle: .alert)

        present(alertController, animated: true) {
            self.logoutSubscriber = Account.shared.logout()
                .delay(for: .seconds(1), scheduler: DispatchQueue.main)
                .sink(receiveCompletion: { (completion) in
                    switch completion {
                    case .failure(let error):
                        alertController.dismiss(animated: true) {
                            self.presentError(error, preferredStyle: .alert)
                        }

                    case .finished:
                        self.performSegue(
                            withIdentifier: SegueIdentifier.Account.logout.rawValue,
                            sender: self)
                    }
                })
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

        copyToPasteboardSubscriber =
            Just(Account.shared.formattedToken)
                .cancellableDelay(for: .seconds(3), scheduler: DispatchQueue.main)
                .sink(receiveValue: { [weak self] (accountToken) in
                    self?.accountTokenButton.setTitle(accountToken, for: .normal)
                })
    }

    @IBAction func doPurchase() {
        guard let product = product else { return }

        let payment = SKPayment(product: product)

        purchaseSubscriber = AppStorePaymentManager.shared
            .addPayment(payment, for: Account.shared.token!)
            .receive(on: DispatchQueue.main)
            .restrictUserInterfaceInteraction(with: compoundInteractionRestriction, animated: true)
            .sink(receiveCompletion: { [weak self] (completion) in
                if case .failure(let error) = completion {
                    self?.presentError(error, preferredStyle: .alert)
                }
                }, receiveValue: { [weak self] (response) in
                    self?.showTimeAddedConfirmationAlert(with: response, context: .purchase)
            })
    }

    @IBAction func restorePurchases() {
        restorePurchasesSubscriber = AppStorePaymentManager.shared
            .restorePurchases(for: Account.shared.token!)
            .receive(on: DispatchQueue.main)
            .restrictUserInterfaceInteraction(with: compoundInteractionRestriction, animated: true)
            .sink(receiveCompletion: { [weak self] (completion) in
                if case .failure(let error) = completion {
                    self?.presentError(error, preferredStyle: .alert)
                }
                }, receiveValue: { [weak self] (response) in
                    self?.showTimeAddedConfirmationAlert(with: response, context: .restoration)
            })
    }

}

private extension SendAppStoreReceiptResponse {

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
            return timeAdded.isZero
                ? NSLocalizedString(
                    "Your previous purchases have already been added to this account.",
                    comment: "")
                : String(
                    format: NSLocalizedString("%@ have been added to your account", comment: ""),
                    formattedTimeAdded ?? "")
        }
    }
}
