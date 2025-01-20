//
//  AccountViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations
import StoreKit
import UIKit

enum AccountViewControllerAction: Sendable {
    case deviceInfo
    case finish
    case logOut
    case navigateToVoucher
    case navigateToDeleteAccount
    case restorePurchasesInfo
}

class AccountViewController: UIViewController, @unchecked Sendable {
    typealias ActionHandler = (AccountViewControllerAction) -> Void

    private let interactor: AccountInteractor
    private let errorPresenter: PaymentAlertPresenter

    private let contentView: AccountContentView = {
        let contentView = AccountContentView()
        contentView.translatesAutoresizingMaskIntoConstraints = false
        return contentView
    }()

    private var productState: ProductState = .none
    private var paymentState: PaymentState = .none

    var actionHandler: ActionHandler?

    init(interactor: AccountInteractor, errorPresenter: PaymentAlertPresenter) {
        self.interactor = interactor
        self.errorPresenter = errorPresenter

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - View lifecycle

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .secondaryColor

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "Account",
            value: "Account",
            comment: ""
        )

        navigationItem.rightBarButtonItem = UIBarButtonItem(
            barButtonSystemItem: .done,
            target: self,
            action: #selector(handleDismiss)
        )

        contentView.accountTokenRowView.copyAccountNumber = { [weak self] in
            self?.copyAccountToken()
        }

        contentView.accountDeviceRow.infoButtonAction = { [weak self] in
            self?.actionHandler?(.deviceInfo)
        }

        contentView.restorePurchasesView.restoreButtonAction = { [weak self] in
            self?.restorePurchases()
        }

        contentView.restorePurchasesView.infoButtonAction = { [weak self] in
            self?.actionHandler?(.restorePurchasesInfo)
        }

        interactor.didReceiveDeviceState = { [weak self] deviceState in
            Task { @MainActor in
                self?.updateView(from: deviceState)
            }
        }

        interactor.didReceivePaymentEvent = { [weak self] event in
            Task { @MainActor in
                self?.didReceivePaymentEvent(event)
            }
        }
        configUI()
        addActions()
        updateView(from: interactor.deviceState)
        applyViewState(animated: false)
        requestStoreProductsIfCan()
    }

    // MARK: - Private

    private func requestStoreProductsIfCan() {
        if StorePaymentManager.canMakePayments {
            requestStoreProducts()
        } else {
            setProductState(.cannotMakePurchases, animated: false)
        }
    }

    private func configUI() {
        let scrollView = UIScrollView()

        view.addConstrainedSubviews([scrollView]) {
            scrollView.pinEdgesToSuperview()
        }

        scrollView.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview(.all().excluding(.bottom))
            contentView.bottomAnchor.constraint(greaterThanOrEqualTo: scrollView.safeAreaLayoutGuide.bottomAnchor)
            contentView.widthAnchor.constraint(equalTo: scrollView.widthAnchor)
        }
    }

    private func addActions() {
        contentView.redeemVoucherButton.addTarget(
            self,
            action: #selector(redeemVoucher),
            for: .touchUpInside
        )

        contentView.purchaseButton.addTarget(
            self,
            action: #selector(doPurchase),
            for: .touchUpInside
        )

        contentView.logoutButton.addTarget(self, action: #selector(logOut), for: .touchUpInside)

        contentView.deleteButton.addTarget(self, action: #selector(deleteAccount), for: .touchUpInside)

        contentView.storeKit2Button.addTarget(self, action: #selector(handleStoreKit2Purchase), for: .touchUpInside)
    }

    private func requestStoreProducts() {
        let productKind = StoreSubscription.thirtyDays

        setProductState(.fetching(productKind), animated: true)

        _ = interactor.requestProducts(with: [productKind]) { [weak self] completion in
            let productState: ProductState = completion.value?.products.first
                .map { .received($0) } ?? .failed

            /// `@MainActor` isolation is safe here because
            /// `ProductsRequestOperation` sets its `completionQueue` to `.main`
            MainActor.assumeIsolated {
                self?.setProductState(productState, animated: true)
            }
        }
    }

    @MainActor
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
        contentView.accountDeviceRow.setButtons(enabled: isInteractionEnabled)
        contentView.accountTokenRowView.setButtons(enabled: isInteractionEnabled)
        contentView.restorePurchasesView.setButtons(enabled: isInteractionEnabled)
        contentView.logoutButton.isEnabled = isInteractionEnabled
        contentView.redeemVoucherButton.isEnabled = isInteractionEnabled
        contentView.deleteButton.isEnabled = isInteractionEnabled
        contentView.storeKit2Button.isEnabled = isInteractionEnabled
        navigationItem.rightBarButtonItem?.isEnabled = isInteractionEnabled

        view.isUserInteractionEnabled = isInteractionEnabled
        isModalInPresentation = !isInteractionEnabled

        navigationItem.setHidesBackButton(!isInteractionEnabled, animated: animated)
    }

    private func didReceivePaymentEvent(_ event: StorePaymentEvent) {
        guard case let .makingPayment(payment) = paymentState,
              payment == event.payment else { return }

        switch event {
        case let .finished(completion):
            errorPresenter.showAlertForResponse(completion.serverResponse, context: .purchase)

        case let .failure(paymentFailure):
            switch paymentFailure.error {
            case .storePayment(SKError.paymentCancelled):
                break

            default:
                errorPresenter.showAlertForError(paymentFailure.error, context: .purchase)
            }
        }

        setPaymentState(.none, animated: true)
    }

    private func copyAccountToken() {
        guard let accountData = interactor.deviceState.accountData else {
            return
        }

        UIPasteboard.general.string = accountData.number
    }

    // MARK: - Actions

    @objc private func logOut() {
        actionHandler?(.logOut)
    }

    @objc private func handleDismiss() {
        actionHandler?(.finish)
    }

    @objc private func redeemVoucher() {
        actionHandler?(.navigateToVoucher)
    }

    @objc private func deleteAccount() {
        actionHandler?(.navigateToDeleteAccount)
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
            guard let self else { return }

            Task { @MainActor in
                switch completion {
                case let .success(response):
                    errorPresenter.showAlertForResponse(response, context: .restoration)

                case let .failure(error as StorePaymentManagerError):
                    errorPresenter.showAlertForError(error, context: .restoration)

                default:
                    break
                }

                setPaymentState(.none, animated: true)
            }
        }
    }

    @objc private func handleStoreKit2Purchase() {
        guard case let .received(oldProduct) = productState,
              let accountData = interactor.deviceState.accountData
        else {
            return
        }

        setPaymentState(.makingStoreKit2Purchase, animated: true)

        Task {
            do {
                let product = try await Product.products(for: [oldProduct.productIdentifier]).first!
                let result = try await product.purchase()

                switch result {
                case let .success(verification):
                    let transaction = try checkVerified(verification)
                    await sendReceiptToAPI(accountNumber: accountData.identifier, receipt: verification)
                    await transaction.finish()

                case .userCancelled:
                    print("User cancelled the purchase")
                case .pending:
                    print("Purchase is pending")
                @unknown default:
                    print("Unknown purchase result")
                }
            } catch {
                print("Error: \(error)")
                errorPresenter.showAlertForStoreKitError(error, context: .purchase)
            }

            setPaymentState(.none, animated: true)
        }
    }

    private func checkVerified<T>(_ result: VerificationResult<T>) throws -> T {
        switch result {
        case .unverified:
            throw StoreKit2Error.verificationFailed
        case let .verified(safe):
            return safe
        }
    }

    private func sendReceiptToAPI(accountNumber: String, receipt: VerificationResult<Transaction>) async {
        do {
            try await interactor.sendStoreKitReceipt(receipt, for: accountNumber)
            print("Receipt sent successfully")
        } catch {
            print("Error sending receipt: \(error)")
            errorPresenter.showAlertForStoreKitError(error, context: .purchase)
        }
    }
}

private enum StoreKit2Error: Error {
    case verificationFailed
}
