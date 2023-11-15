//
//  OutOfTimeViewController.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-07-25.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import Operations
import StoreKit
import UIKit

protocol OutOfTimeViewControllerDelegate: AnyObject {
    func outOfTimeViewControllerDidBeginPayment(_ controller: OutOfTimeViewController)
    func outOfTimeViewControllerDidEndPayment(_ controller: OutOfTimeViewController)
}

class OutOfTimeViewController: UIViewController, RootContainment {
    weak var delegate: OutOfTimeViewControllerDelegate?

    private let interactor: OutOfTimeInteractor
    private let errorPresenter: PaymentAlertPresenter

    private var productState: ProductState = .none {
        didSet {
            applyViewState()
        }
    }

    private var paymentState: PaymentState = .none {
        didSet {
            applyViewState()
            notifyDelegate(oldValue)
        }
    }

    private lazy var contentView = OutOfTimeContentView()

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        let tunnelState = interactor.tunnelStatus.state

        return HeaderBarPresentation(
            style: tunnelState.isSecured ? .secured : .unsecured,
            showsDivider: false
        )
    }

    var prefersHeaderBarHidden: Bool {
        false
    }

    init(interactor: OutOfTimeInteractor, errorPresenter: PaymentAlertPresenter) {
        self.interactor = interactor
        self.errorPresenter = errorPresenter

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])

        contentView.disconnectButton.addTarget(
            self,
            action: #selector(handleDisconnect(_:)),
            for: .touchUpInside
        )
        contentView.purchaseButton.addTarget(
            self,
            action: #selector(doPurchase),
            for: .touchUpInside
        )
        contentView.restoreButton.addTarget(
            self,
            action: #selector(restorePurchases),
            for: .touchUpInside
        )

        interactor.didReceivePaymentEvent = { [weak self] event in
            self?.didReceivePaymentEvent(event)
        }

        interactor.didReceiveTunnelStatus = { [weak self] _ in
            self?.setNeedsHeaderBarStyleAppearanceUpdate()
            self?.applyViewState()
        }

        if StorePaymentManager.canMakePayments {
            requestStoreProducts()
        } else {
            productState = .cannotMakePurchases
        }
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        interactor.startAccountUpdateTimer()
    }

    override func viewDidDisappear(_ animated: Bool) {
        super.viewDidDisappear(animated)
        interactor.stopAccountUpdateTimer()
    }

    // MARK: - Private

    private func requestStoreProducts() {
        let productKind = StoreSubscription.thirtyDays

        productState = .fetching(productKind)

        _ = interactor.requestProducts(with: [productKind]) { [weak self] completion in
            let productState: ProductState = completion.value?.products.first
                .map { .received($0) } ?? .failed

            self?.productState = productState
        }
    }

    private func applyViewState() {
        let tunnelState = interactor.tunnelStatus.state
        let isInteractionEnabled = paymentState.allowsViewInteraction
        let purchaseButton = contentView.purchaseButton

        let isOutOfTime = interactor.deviceState.accountData.map { $0.expiry < Date() } ?? false

        purchaseButton.setTitle(productState.purchaseButtonTitle, for: .normal)
        contentView.purchaseButton.isLoading = productState.isFetching

        purchaseButton.isEnabled = productState.isReceived && isInteractionEnabled && !tunnelState
            .isSecured
        contentView.restoreButton.isEnabled = isInteractionEnabled

        contentView.enableDisconnectButton(tunnelState.isSecured, animated: true)

        if tunnelState.isSecured {
            contentView.setBodyLabelText(
                NSLocalizedString(
                    "OUT_OF_TIME_BODY_CONNECTED",
                    tableName: "OutOfTime",
                    value: """
                    You have no more VPN time left on this account. To add more, you will need to \
                    disconnect and access the Internet with an unsecure connection.
                    """,
                    comment: ""
                )
            )
        } else {
            contentView.setBodyLabelText(
                NSLocalizedString(
                    "OUT_OF_TIME_BODY_DISCONNECTED",
                    tableName: "OutOfTime",
                    value: """
                    You have no more VPN time left on this account. Either buy credit on our website \
                    or make an in-app purchase via the **Add 30 days time** button below.
                    """,
                    comment: ""
                )
            )
        }

        if !isInteractionEnabled {
            contentView.statusActivityView.state = .activity
        } else {
            contentView.statusActivityView.state = isOutOfTime ? .failure : .success
        }

        view.isUserInteractionEnabled = isInteractionEnabled
    }

    private func notifyDelegate(_ oldPaymentState: PaymentState) {
        switch (oldPaymentState, paymentState) {
        case (.none, .makingPayment), (.none, .restoringPurchases):
            delegate?.outOfTimeViewControllerDidBeginPayment(self)

        case (.makingPayment, .none), (.restoringPurchases, .none):
            delegate?.outOfTimeViewControllerDidEndPayment(self)

        default:
            break
        }
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
                errorPresenter.showAlertForError(paymentFailure.error, context: .purchase) {
                    self.paymentState = .none
                }
            }
        }

        paymentState = .none
    }

    // MARK: - Actions

    @objc private func doPurchase() {
        guard case let .received(product) = productState,
              let accountData = interactor.deviceState.accountData
        else {
            return
        }

        let payment = SKPayment(product: product)
        interactor.addPayment(payment, for: accountData.number)

        paymentState = .makingPayment(payment)
    }

    @objc func restorePurchases() {
        guard let accountData = interactor.deviceState.accountData else {
            return
        }

        paymentState = .restoringPurchases

        _ = interactor.restorePurchases(for: accountData.number) { [weak self] result in
            guard let self else { return }

            switch result {
            case let .success(response):
                errorPresenter.showAlertForResponse(response, context: .restoration) {
                    self.paymentState = .none
                }

            case let .failure(error as StorePaymentManagerError):
                errorPresenter.showAlertForError(error, context: .restoration) {
                    self.paymentState = .none
                }

            default:
                paymentState = .none
            }
        }
    }

    @objc private func handleDisconnect(_ sender: Any) {
        contentView.disconnectButton.isEnabled = false
        interactor.stopTunnel()
    }
}
