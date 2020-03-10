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
            self?.purchaseButton.isEnabled = enableUserInteraction
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

        // Make sure the buy button scales down the font size to fit the long labels.
        // Changing baseline adjustment helps to prevent the text from being misaligned after
        // being scaled down.
        purchaseButton.titleLabel?.adjustsFontSizeToFitWidth = true
        purchaseButton.titleLabel?.baselineAdjustment = .alignCenters

        accountTokenButton.setTitle(Account.shared.token, for: .normal)

        if let expiryDate = Account.shared.expiry {
            updateAccountExpiry(expiryDate: expiryDate)
        }

        requestStoreProducts()
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
        purchaseButton.isLoading = true

        requestProductsSubscriber = AppStorePaymentManager.shared.requestProducts(with: [.thirtyDays])
            .retry(1)
            .receive(on: DispatchQueue.main)
            .restrictUserInterfaceInteraction(with: self.purchaseButtonInteractionRestriction, animated: true)
            .sink(receiveCompletion: { [weak self] (completion) in
                if case .finished = completion {
                    self?.purchaseButton.isLoading = false
                }
                }, receiveValue: { [weak self] (response) in
                    if let product = response.products.first {
                        self?.setProduct(product, animated: true)
                    }
            })
    }

    private func setProduct(_ product: SKProduct, animated: Bool) {
        self.product = product

        let localizedPrice = product.localizedPrice ?? ""
        let title = String(format: NSLocalizedString("%@ (%@)", comment: ""),
                           product.localizedTitle, localizedPrice)
        purchaseButton.setTitle(title, for: .normal)
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

    // MARK: - Actions

    @IBAction func doLogout() {
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
                        self.performSegue(withIdentifier: SegueIdentifier.Account.logout.rawValue, sender: self)
                    }
                })
        }
    }

    @IBAction func copyAccountToken() {
        let accountToken = Account.shared.token

        UIPasteboard.general.string = accountToken

        accountTokenButton.setTitle(
            NSLocalizedString("COPIED TO PASTEBOARD!", comment: ""),
            for: .normal)

        copyToPasteboardSubscriber =
            Just(accountToken).cancellableDelay(for: .seconds(3), scheduler: DispatchQueue.main)
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
