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
        switch paymentAction {
        case .purchase:
            #if DEBUG
                startPaymentFlow()
            #else
                startOldPaymentFlow()
            #endif
        case .restorePurchase:
            _ = storePaymentManager.restorePurchases(for: accountNumber) { result in
                Task { @MainActor [weak self] in
                    guard let self else { return }
                    self.spinnerView.stopAnimating()
                    switch result {
                    case let .success(success):
                        self.errorPresenter.showAlertForResponse(success, context: .restoration) {
                            self.didFinish?()
                        }
                    case let .failure(failure as StorePaymentManagerError):
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

    // Store Kit 2
    func startPaymentFlow() {
        Task { @MainActor [weak self] in
            guard let self else { return }
            var products: [Product]
            do {
                products = try await storePaymentManager.products()
            } catch {
                self.didFinish?()
                self.spinnerView.stopAnimating()
                return
            }
            self.spinnerView.stopAnimating()
            guard !products.isEmpty else {
                return
            }
            self.showPurchaseOptions(for: products)
        }
    }

    // Store Kit 2
    func purchase(product: Product) async throws {
        await storePaymentManager.purchase(product: product, for: accountNumber)
    }

    // Store Kit 2
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
                                try await self.purchase(product: product)
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

    func startOldPaymentFlow() {
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
                    self.oldShowPurchaseOptions(for: products)
                case let .failure(failure as StorePaymentManagerError):
                    self.errorPresenter.showAlertForError(failure, context: .purchase) {
                        self.didFinish?()
                    }
                case .failure:
                    self.didFinish?()
                }
            }
        }
    }

    func oldShowPurchaseOptions(for products: [SKProduct]) {
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
                            self.purchase(product: product)
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

    func purchase(product: SKProduct) {
        let payment = SKPayment(product: product)
        storePaymentManager.addPayment(payment, for: accountNumber)
    }

    nonisolated func storePaymentManager(_ manager: StorePaymentManager, didReceiveEvent event: StorePaymentEvent) {
        Task { @MainActor in
            spinnerView.stopAnimating()
            switch event {
            case let .finished(completion):
                errorPresenter.showAlertForResponse(completion.serverResponse, context: .purchase) {
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

    nonisolated func storePaymentManager(_ manager: StorePaymentManager, didReceiveEvent event: StoreKitPaymentEvent) {
        Task { @MainActor in
            spinnerView.stopAnimating()
            switch event {
            case .successfulPayment, .pending, .userCancelled:
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
}
