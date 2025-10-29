//
//  SheetViewController.swift
//  MullvadVPN
//
//  Created by Steffen Ernst on 2025-01-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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
        self.storePaymentManager.addPaymentObserver(self)
        modalPresentationStyle = .overFullScreen
        modalTransitionStyle = .crossDissolve
        view.addConstrainedSubviews([spinnerView]) {
            spinnerView.centerXAnchor.constraint(equalTo: view.centerXAnchor)
            spinnerView.centerYAnchor.constraint(equalTo: view.centerYAnchor)
        }
        view.backgroundColor = .black.withAlphaComponent(0.5)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        spinnerView.startAnimating()

        #if DEBUG
            handlePaymentAction(paymentAction)
        #else
            handleLegacyPaymentAction(paymentAction)
        #endif
    }

    // MARK: StoreKit 2 flow

    func handlePaymentAction(_ action: PaymentAction) {
        switch action {
        case .purchase:
            startPaymentFlow()
        case .restorePurchase:
            Task {
                let outcome = (try? await storePaymentManager.processUnfinishedTransactions()) ?? .noTimeAdded
                errorPresenter.showAlertForOutcome(outcome, context: .restoration) {
                    self.didFinish?()
                }
            }
        }
    }

    func startPaymentFlow() {
        Task {
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
    }

    func showPurchaseOptions(for products: [Product]) {
        let localizedString = NSLocalizedString("Add Time", comment: "")
        let sheetController = UIAlertController(
            title: localizedString,
            message: nil,
            preferredStyle: UIDevice.current.userInterfaceIdiom == .pad ? .alert : .actionSheet
        )
        sheetController.overrideUserInterfaceStyle = .dark
        sheetController.view.tintColor = .AlertController.tintColor
        products.sorted { $0.price < $1.price }.forEach { product in
            let title = product.displayName
            let action = UIAlertAction(
                title: title, style: .default,
                handler: { _ in
                    sheetController.dismiss(
                        animated: true,
                        completion: {
                            self.spinnerView.startAnimating()
                            Task { @MainActor in
                                await self.storePaymentManager.purchase(product: product, for: self.accountNumber)
                            }
                        })
                })
            action
                .accessibilityIdentifier = action.accessibilityIdentifier
            sheetController.addAction(action)
        }

        let cancelAction = UIAlertAction(title: NSLocalizedString("Cancel", comment: ""), style: .cancel) { _ in
            self.didFinish?()
        }
        cancelAction.accessibilityIdentifier = "actoin-sheet-cancel-button"
        sheetController.addAction(cancelAction)
        present(sheetController, animated: true)
    }

    nonisolated func storePaymentManager(didReceiveEvent event: StorePaymentEvent) {
        Task { @MainActor in
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

    // MARK: Legacy StoreKit flow

    func handleLegacyPaymentAction(_ action: PaymentAction) {
        switch paymentAction {
        case .purchase:
            startLegacyPaymentFlow()
        case .restorePurchase:
            _ = storePaymentManager.restorePurchases(for: accountNumber) { result in
                Task { @MainActor [weak self] in
                    guard let self else { return }
                    self.spinnerView.stopAnimating()
                    switch result {
                    case let .success(success):
                        let outcome = StorePaymentOutcome.timeAdded(success.timeAdded)
                        self.errorPresenter.showAlertForOutcome(outcome, context: .restoration) {
                            self.didFinish?()
                        }
                    case let .failure(failure as LegacyStorePaymentManagerError):
                        self.errorPresenter.showAlertForError(failure, context: .restoration) {
                            self.didFinish?()
                        }
                    case .failure:
                        self.didFinish?()
                    }
                }
            }
        }
    }

    func startLegacyPaymentFlow() {
        let productIdentifiers = Set(StoreSubscription.allCases)

        _ = storePaymentManager.requestProducts(
            with: productIdentifiers
        ) { result in
            Task { @MainActor [weak self] in
                guard let self else { return }
                self.spinnerView.stopAnimating()
                switch result {
                case let .success(success):
                    let products = success.products
                    guard !products.isEmpty else {
                        return
                    }
                    self.LegacyShowPurchaseOptions(for: products)
                case let .failure(failure as LegacyStorePaymentManagerError):
                    self.errorPresenter.showAlertForError(failure, context: .purchase) {
                        self.didFinish?()
                    }
                case .failure:
                    self.didFinish?()
                }
            }
        }
    }

    func LegacyShowPurchaseOptions(for products: [SKProduct]) {
        let localizedString = NSLocalizedString("Add Time", comment: "")
        let sheetController = UIAlertController(
            title: localizedString,
            message: nil,
            preferredStyle: UIDevice.current.userInterfaceIdiom == .pad ? .alert : .actionSheet
        )
        sheetController.overrideUserInterfaceStyle = .dark
        sheetController.view.tintColor = .AlertController.tintColor
        products.sortedByPrice().forEach { product in
            guard let title = product.customLocalizedTitle else { return }
            let action = UIAlertAction(
                title: title, style: .default,
                handler: { _ in
                    sheetController.dismiss(
                        animated: true,
                        completion: {
                            self.storePaymentManager.addPayment(SKPayment(product: product), for: self.accountNumber)
                            self.spinnerView.startAnimating()
                        })
                })
            action
                .accessibilityIdentifier = action.accessibilityIdentifier
            sheetController.addAction(action)
        }

        let cancelAction = UIAlertAction(title: NSLocalizedString("Cancel", comment: ""), style: .cancel) { _ in
            self.didFinish?()
        }
        cancelAction.accessibilityIdentifier = "actoin-sheet-cancel-button"
        sheetController.addAction(cancelAction)
        present(sheetController, animated: true)
    }

    nonisolated func storePaymentManager(didReceiveEvent event: LegacyStorePaymentEvent) {
        Task { @MainActor in
            spinnerView.stopAnimating()
            switch event {
            case let .finished(completion):
                let outcome = StorePaymentOutcome.timeAdded(completion.serverResponse.timeAdded)
                errorPresenter.showAlertForOutcome(outcome, context: .purchase) {
                    self.didFinish?()
                }

            case let .failure(paymentFailure):
                switch paymentFailure.error {
                case .storePayment(SKError.paymentCancelled):
                    self.didFinish?()
                default:
                    errorPresenter.showAlertForError(paymentFailure.error, context: .purchase) {
                        self.didFinish?()
                    }
                }
            }
        }
    }
}
