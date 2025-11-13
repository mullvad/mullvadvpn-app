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
    case deviceManagement
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
        return contentView
    }()

    private var isFetchingProducts = false
    private var paymentState: PaymentState = .none
    private let storeKit2TestProduct = LegacyStoreSubscription.thirtyDays.rawValue

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

        navigationItem.title = NSLocalizedString("Account", comment: "")

        navigationItem.rightBarButtonItem = UIBarButtonItem(
            barButtonSystemItem: .done,
            target: self,
            action: #selector(handleDismiss)
        )

        contentView.accountTokenRowView.copyAccountNumber = { [weak self] in
            self?.copyAccountToken()
        }

        contentView.accountDeviceRow.deviceManagementButtonAction = { [weak self] in
            self?.actionHandler?(.deviceManagement)
        }

        contentView.restorePurchasesView.restoreButtonAction = { [weak self] in
            self?.restorePurchases()
        }

        contentView.restorePurchasesView.infoButtonAction = { [weak self] in
            self?.actionHandler?(.restorePurchasesInfo)
        }

        interactor.didReceiveTunnelState = { [weak self] in
            guard let self else { return }
            Task { @MainActor in
                applyViewState(animated: true)
            }
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
        view.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview()
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

    private func updateView(from deviceState: DeviceState) {
        guard case let .loggedIn(accountData, deviceData) = deviceState else {
            return
        }

        contentView.accountDeviceRow.deviceName = deviceData.name
        contentView.accountTokenRowView.accountNumber = accountData.number
        contentView.accountExpiryRowView.value = accountData.expiry
    }

    private func applyViewState(animated: Bool) {
        let isInteractionEnabled = paymentState.allowsViewInteraction

        contentView.purchaseButton.isEnabled =
            !isFetchingProducts
            && isInteractionEnabled
            && !interactor.tunnelState.isBlockingInternet
        contentView.accountDeviceRow.setButtons(enabled: isInteractionEnabled)
        contentView.accountTokenRowView.setButtons(enabled: isInteractionEnabled)
        contentView.restorePurchasesView.setButtons(enabled: isInteractionEnabled)
        contentView.logoutButton.isEnabled = isInteractionEnabled
        contentView.redeemVoucherButton.isEnabled = isInteractionEnabled
        contentView.deleteButton.isEnabled = isInteractionEnabled
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

    // For testing StoreKit 2 refunds only.
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
                errorPresenter.showAlertForRefundError(error, context: .purchase)
            }

            setPaymentState(.none, animated: true)
        }
    }
}
