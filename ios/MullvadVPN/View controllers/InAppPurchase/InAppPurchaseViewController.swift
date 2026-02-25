//
//  SheetViewController.swift
//  MullvadVPN
//
//  Created by Steffen Ernst on 2025-01-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import StoreKit
import UIKit

class InAppPurchaseViewController: UIViewController, StorePaymentObserver {
    private let storePaymentManager: StorePaymentManager
    private let accountNumber: String
    private let paymentAction: PaymentAction
    private let errorPresenter: PaymentAlertPresenter

    private let spinnerView = {
        SpinnerActivityIndicatorView(style: .large)
    }()

    var didFinish: (() -> Void)?

    init(
        storePaymentManager: StorePaymentManager,
        accountNumber: String,
        errorPresenter: PaymentAlertPresenter,
        paymentAction: PaymentAction
    ) {
        self.storePaymentManager = storePaymentManager
        self.accountNumber = accountNumber
        self.errorPresenter = errorPresenter
        self.paymentAction = paymentAction

        super.init(nibName: nil, bundle: nil)

        Task {
            await storePaymentManager.addPaymentObserver(self)
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        modalPresentationStyle = .overFullScreen
        modalTransitionStyle = .crossDissolve

        view.backgroundColor = .black.withAlphaComponent(0.5)
        view.addConstrainedSubviews([spinnerView]) {
            spinnerView.centerXAnchor.constraint(equalTo: view.centerXAnchor)
            spinnerView.centerYAnchor.constraint(equalTo: view.centerYAnchor)
        }

        Task {
            await handlePaymentAction(paymentAction)

            // NOTE! When enabling or disabling legacy payments, make sure
            // to also enable/disable them in StorePaymentManager.start().
            // await handleLegacyPaymentAction(paymentAction)
        }
    }

    func handlePaymentAction(_ action: PaymentAction) async {
        switch action {
        case .purchase:
            await startRestorationBeforePurchaseFlow()
        case .restorePurchase:
            spinnerView.startAnimating()

            do {
                let outcome = try await storePaymentManager.processOutstandingTransactions()
                spinnerView.stopAnimating()
                errorPresenter.showAlertForOutcome(outcome, context: .restoration) {
                    self.didFinish?()
                }
            } catch {
                spinnerView.stopAnimating()
                errorPresenter.showAlertForError(.restorationError, context: .restoration) {
                    self.didFinish?()
                }
            }
        }
    }

    func startRestorationBeforePurchaseFlow() async {
        spinnerView.startAnimating()

        do {
            let outcome = try await storePaymentManager.processOutstandingTransactions()
            spinnerView.stopAnimating()

            if case .timeAdded = outcome {
                await withCheckedContinuation { continuation in
                    errorPresenter.showAlertForOutcome(outcome, context: .restorationBeforePurchase) {
                        continuation.resume()
                    }
                }
            }

            await startPaymentFlow()
        } catch {
            spinnerView.stopAnimating()

            errorPresenter.showAlertForError(.restorationError, context: .purchase) {
                self.didFinish?()
            }
        }
    }

    func startPaymentFlow() async {
        spinnerView.startAnimating()

        var products: [Product]
        do {
            products = try await storePaymentManager.products()
        } catch {
            spinnerView.stopAnimating()
            didFinish?()
            return
        }

        spinnerView.stopAnimating()

        guard !products.isEmpty else {
            return
        }

        showPurchaseOptions(for: products)
    }

    func showPurchaseOptions(for products: [Product]) {
        let localizedString = NSLocalizedString("Add time", comment: "")

        let sheetController = UIAlertController(
            title: localizedString,
            message: nil,
            preferredStyle: UIDevice.current.userInterfaceIdiom == .pad ? .alert : .actionSheet
        )
        sheetController.overrideUserInterfaceStyle = .dark
        sheetController.view.tintColor = .AlertController.tintColor

        products.sorted { $0.price < $1.price }.forEach { product in
            guard let title = product.customLocalizedTitle else { return }

            let action = UIAlertAction(
                title: title, style: .default,
                handler: { _ in
                    sheetController.dismiss(
                        animated: true,
                        completion: {
                            self.spinnerView.startAnimating()

                            Task {
                                await self.storePaymentManager.purchase(product: product)
                            }
                        }
                    )
                }
            )

            sheetController.addAction(action)
        }

        let cancelAction = UIAlertAction(title: NSLocalizedString("Cancel", comment: ""), style: .cancel) { _ in
            self.didFinish?()
        }
        cancelAction.accessibilityIdentifier = "action-sheet-cancel-button"

        sheetController.addAction(cancelAction)

        present(sheetController, animated: true)
    }

    @MainActor
    func storePaymentManager(didReceiveEvent event: StorePaymentEvent) {
        spinnerView.stopAnimating()

        switch event {
        case let .successfulPayment(outcome):
            errorPresenter.showAlertForOutcome(outcome, context: .purchase) {
                self.didFinish?()
            }
        case .pending, .userCancelled:
            self.didFinish?()
        case let .failed(error):
            errorPresenter.showAlertForError(error, context: .purchase) {
                self.didFinish?()
            }
        @unknown default:
            self.didFinish?()
        }
    }
}
