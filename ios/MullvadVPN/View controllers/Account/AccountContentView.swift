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
        button.translatesAutoresizingMaskIntoConstraints = false
        button.accessibilityIdentifier = "PurchaseButton"
        return button
    }()

    let restorePurchasesButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString(
            "RESTORE_PURCHASES_BUTTON_TITLE",
            tableName: "Account",
            value: "Restore purchases",
            comment: ""
        ), for: .normal)
        return button
    }()

    let redeemVoucherButton: AppButton = {
        let button = AppButton(style: .success)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.accessibilityIdentifier = "redeemVoucherButton"
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
        button.translatesAutoresizingMaskIntoConstraints = false
        button.accessibilityIdentifier = "LogoutButton"
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
        button.translatesAutoresizingMaskIntoConstraints = false
        button.accessibilityIdentifier = "DeleteButton"
        button.setTitle(NSLocalizedString(
            "DELETE_BUTTON_TITLE",
            tableName: "Account",
            value: "Delete account",
            comment: ""
        ), for: .normal)
        return button
    }()

    let accountDeviceRow: AccountDeviceRow = {
        let view = AccountDeviceRow()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    let accountTokenRowView: AccountNumberRow = {
        let view = AccountNumberRow()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    let accountExpiryRowView: AccountExpiryRow = {
        let view = AccountExpiryRow()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    lazy var contentStackView: UIStackView = {
        let stackView =
            UIStackView(arrangedSubviews: [
                accountDeviceRow,
                accountTokenRowView,
                accountExpiryRowView,
            ])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.padding8
        return stackView
    }()

    lazy var buttonStackView: UIStackView = {
        var arrangedSubviews = [UIView]()
        #if DEBUG
        arrangedSubviews.append(redeemVoucherButton)
        #endif
        arrangedSubviews.append(contentsOf: [
            purchaseButton,
            restorePurchasesButton,
            logoutButton,
            deleteButton,
        ])
        let stackView = UIStackView(arrangedSubviews: arrangedSubviews)
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.padding16
        stackView.setCustomSpacing(UIMetrics.interButtonSpacing, after: restorePurchasesButton)
        return stackView
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        directionalLayoutMargins = UIMetrics.contentLayoutMargins

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
