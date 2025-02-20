//
//  OutOfTimeViewController.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-07-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import Operations
@preconcurrency import StoreKit
import UIKit

protocol OutOfTimeViewControllerDelegate: AnyObject, Sendable {
    func didRequestShowInAppPurchase(
        accountNumber: String,
        paymentAction: PaymentAction
    )
}

@MainActor
class OutOfTimeViewController: UIViewController, RootContainment {
    weak var delegate: OutOfTimeViewControllerDelegate?

    private let interactor: OutOfTimeInteractor

    private lazy var contentView = OutOfTimeContentView()

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    nonisolated(unsafe) var preferredHeaderBarPresentation: HeaderBarPresentation {
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
            action: #selector(requestStoreProducts),
            for: .touchUpInside
        )
        contentView.restoreButton.addTarget(
            self,
            action: #selector(restorePurchases),
            for: .touchUpInside
        )

        interactor.didReceiveTunnelStatus = { [weak self] _ in
            Task { @MainActor in
                self?.setNeedsHeaderBarStyleAppearanceUpdate()
                self?.applyViewState()
            }
        }
        applyViewState()
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

    private func applyViewState() {
        let tunnelState = interactor.tunnelStatus.state
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
                    or make an in-app purchase via the **Add time** button below.
                    """,
                    comment: ""
                )
            )
        }
    }

    // MARK: - Actions

    @objc private func requestStoreProducts() {
        guard let accountNumber = interactor.deviceState.accountData?.number else {
            return
        }
        delegate?.didRequestShowInAppPurchase(
            accountNumber: accountNumber,
            paymentAction: .purchase
        )
    }

    @objc func restorePurchases() {
        guard let accountNumber = interactor.deviceState.accountData?.number else {
            return
        }
        delegate?.didRequestShowInAppPurchase(
            accountNumber: accountNumber,
            paymentAction: .restorePurchase
        )
    }

    @objc private func handleDisconnect(_ sender: Any) {
        contentView.disconnectButton.isEnabled = false
        interactor.stopTunnel()
    }
}
