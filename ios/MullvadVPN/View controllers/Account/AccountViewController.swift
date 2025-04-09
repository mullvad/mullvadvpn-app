//
//  AccountViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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
    case showPurchaseOptions
    case showFailedToLoadProducts
    case showRestorePurchases
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

    private var isFetchingProducts = false
    private var paymentState: PaymentState = .none
    private let storeKit2TestProduct = StoreSubscription.thirtyDays.rawValue

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

        configUI()
        addActions()
        updateView(from: interactor.deviceState)
        applyViewState(animated: false)
    }

    // MARK: - Private

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
            action: #selector(requestStoreProducts),
            for: .touchUpInside
        )

        contentView.logoutButton.addTarget(self, action: #selector(logOut), for: .touchUpInside)
        contentView.deleteButton.addTarget(self, action: #selector(deleteAccount), for: .touchUpInside)
        contentView.storeKit2PurchaseButton.addTarget(
            self, action: #selector(handleStoreKit2Purchase),
            for: .touchUpInside
        )
        contentView.storeKit2RefundButton.addTarget(
            self, action: #selector(handleStoreKit2Refund),
            for: .touchUpInside
        )
    }

    @MainActor
    private func setPaymentState(_ newState: PaymentState, animated: Bool) {
        paymentState = newState

        applyViewState(animated: animated)
    }

    private func setIsFetchingProducts(_ isFetchingProducts: Bool, animated: Bool = false) {
        self.isFetchingProducts = isFetchingProducts

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

        purchaseButton.isEnabled = !isFetchingProducts && isInteractionEnabled
        contentView.accountDeviceRow.setButtons(enabled: isInteractionEnabled)
        contentView.accountTokenRowView.setButtons(enabled: isInteractionEnabled)
        contentView.restorePurchasesView.setButtons(enabled: isInteractionEnabled)
        contentView.logoutButton.isEnabled = isInteractionEnabled
        contentView.redeemVoucherButton.isEnabled = isInteractionEnabled
        contentView.deleteButton.isEnabled = isInteractionEnabled
        contentView.storeKit2PurchaseButton.isEnabled = isInteractionEnabled
        contentView.storeKit2RefundButton.isEnabled = isInteractionEnabled
        navigationItem.rightBarButtonItem?.isEnabled = isInteractionEnabled

        view.isUserInteractionEnabled = isInteractionEnabled
        isModalInPresentation = !isInteractionEnabled

        navigationItem.setHidesBackButton(!isInteractionEnabled, animated: animated)
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

    @objc private func requestStoreProducts() {
        actionHandler?(.showPurchaseOptions)
    }

    @objc private func restorePurchases() {
        actionHandler?(.showRestorePurchases)
    }

    // This function is for testing only
    @objc private func handleStoreKit2Purchase() {
        guard let accountData = interactor.deviceState.accountData else {
            return
        }

        setPaymentState(.makingStoreKit2Purchase, animated: true)

        Task {
            do {
                let product = try await Product.products(
                    for: [
                        StoreSubscription
                            .thirtyDays.rawValue,
                    ]
                ).first!
                let token = switch await interactor
                    .getPaymentToken(for: accountData.number) {
                case let .success(token):
                    UUID(uuidString: token)!
                case let .failure(error):
                    throw error
                }

                let result = try await product.purchase(
                    options: [.appAccountToken(token)]
                )

                switch result {
                case let .success(verification):
                    let transaction = try checkVerified(verification)
                    await sendReceiptToAPI(
                        accountNumber: accountData.number,
                        receipt: verification
                    )
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

    @objc private func handleStoreKit2Refund() {
        setPaymentState(.makingStoreKit2Refund, animated: true)

        Task {
            guard
                let latestTransactionResult = await Transaction.latest(for: storeKit2TestProduct),
                let windowScene = view.window?.windowScene
            else { return }

            do {
                switch latestTransactionResult {
                case let .verified(transaction):
                    let refundStatus = try await transaction.beginRefundRequest(in: windowScene)

                    switch refundStatus {
                    case .success:
                        print("Refund was successful")
                        errorPresenter.showAlertForRefund()
                    case .userCancelled:
                        print("User cancelled the refund")
                    @unknown default:
                        print("Unknown refund result")
                    }
                case .unverified:
                    print("Transaction is unverified")
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
        switch await interactor.sendStoreKitReceipt(receipt, for: accountNumber) {
        case .success:
            print("Receipt sent successfully")
        case let .failure(error):
            print("Error sending receipt: \(error)")
            errorPresenter.showAlertForStoreKitError(error, context: .purchase)
        }
    }
}

private enum StoreKit2Error: Error {
    case verificationFailed
}
