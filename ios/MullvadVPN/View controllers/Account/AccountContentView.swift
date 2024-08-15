//
//  AccountContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 08/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class AccountContentView: UIView {
    let purchaseButton: InAppPurchaseButton = {
        let button = InAppPurchaseButton()
        button.accessibilityIdentifier = .purchaseButton
        return button
    }()

    let redeemVoucherButton: AppButton = {
        let button = AppButton(style: .success)
        button.accessibilityIdentifier = .redeemVoucherButton
        button.setTitle(NSLocalizedString(
            "REDEEM_VOUCHER_BUTTON_TITLE",
            tableName: "Account",
            value: "Redeem voucher",
            comment: ""
        ), for: .normal)
        return button
    }()

    let logoutButton: AppButton = {
        let button = AppButton(style: .danger)
        button.accessibilityIdentifier = .logoutButton
        button.setTitle(NSLocalizedString(
            "LOGOUT_BUTTON_TITLE",
            tableName: "Account",
            value: "Log out",
            comment: ""
        ), for: .normal)
        return button
    }()

    let deleteButton: AppButton = {
        let button = AppButton(style: .danger)
        button.accessibilityIdentifier = .deleteButton
        button.setTitle(NSLocalizedString(
            "DELETE_BUTTON_TITLE",
            tableName: "Account",
            value: "Delete account",
            comment: ""
        ), for: .normal)
        return button
    }()

    let accountDeviceRow: AccountDeviceRow = {
        AccountDeviceRow()
    }()

    let accountTokenRowView: AccountNumberRow = {
        AccountNumberRow()
    }()

    let accountExpiryRowView: AccountExpiryRow = {
        AccountExpiryRow()
    }()

    let restorePurchasesView: RestorePurchasesView = {
        RestorePurchasesView()
    }()

    lazy var contentStackView: UIStackView = {
        let stackView =
            UIStackView(arrangedSubviews: [
                accountDeviceRow,
                accountTokenRowView,
                accountExpiryRowView,
                restorePurchasesView,
            ])
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.padding24
        stackView.setCustomSpacing(UIMetrics.padding8, after: accountExpiryRowView)
        return stackView
    }()

    lazy var buttonStackView: UIStackView = {
        var arrangedSubviews = [UIView]()
        #if DEBUG
        arrangedSubviews.append(redeemVoucherButton)
        #endif
        arrangedSubviews.append(contentsOf: [
            purchaseButton,
            logoutButton,
            deleteButton,
        ])
        let stackView = UIStackView(arrangedSubviews: arrangedSubviews)
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.padding16
        return stackView
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        directionalLayoutMargins = UIMetrics.contentLayoutMargins
        accessibilityIdentifier = .accountView

        addConstrainedSubviews([contentStackView, buttonStackView]) {
            contentStackView.pinEdgesToSuperviewMargins(.all().excluding(.bottom))
            buttonStackView.topAnchor.constraint(
                greaterThanOrEqualTo: contentStackView.bottomAnchor,
                constant: UIMetrics.TableView.sectionSpacing
            )
            buttonStackView.pinEdgesToSuperviewMargins(.all().excluding(.top))
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
