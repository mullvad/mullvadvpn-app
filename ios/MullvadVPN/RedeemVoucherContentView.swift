//
//  RedeemVoucherContentView.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-05.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class RedeemVoucherContentView: UIView {
    let titleLabel: UILabel = {
        let label = UILabel()
        label.text = NSLocalizedString(
            "REDEEM_VOUCHER_TITLE",
            tableName: "RedeemVoucher",
            value: "Redeem Voucher",
            comment: ""
        )
        label.font = UIFont.systemFont(ofSize: 32)
        label.textColor = .white
        return label
    }()

    let instructionLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 17)
        label.text = NSLocalizedString(
            "REDEEM_VOUCHER_INSTRUCTION",
            tableName: "RedeemVoucher",
            value: "Enter voucher code",
            comment: ""
        )
        label.textColor = UIColor.white.withAlphaComponent(0.6)
        label.translatesAutoresizingMaskIntoConstraints = false
        label.numberOfLines = 0
        return label
    }()

    let inputTextField: VoucherTextField = {
        let textField = VoucherTextField()
        textField.font = UIFont.backport_monospacedSystemFont(ofSize: 20, weight: .regular)
        textField.translatesAutoresizingMaskIntoConstraints = false
        textField.placeholder = "XXXX-XXXX-XXXX-XXXX"
        textField.placeholderTextColor = .lightGray
        textField.backgroundColor = .white
        textField.cornerRadius = 8
        textField.keyboardType = .default
        textField.autocapitalizationType = .allCharacters
        textField.returnKeyType = .done

        return textField
    }()

    let activityIndicator: SpinnerActivityIndicatorView = {
        let activityIndicator = SpinnerActivityIndicatorView(style: .medium)
        activityIndicator.translatesAutoresizingMaskIntoConstraints = false
        activityIndicator.tintColor = .white
        return activityIndicator
    }()

    let statusLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 17)
        label.textColor = UIColor.white.withAlphaComponent(0.6)
        label.translatesAutoresizingMaskIntoConstraints = false
        label.numberOfLines = 0
        label.alpha = 0
        return label
    }()

    let redeemButton: AppButton = {
        let button = AppButton(style: .success)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString(
            "REDEEM_VOUCHER_REDEEM_BUTTON",
            tableName: "RedeemVoucher",
            value: "Redeem",
            comment: ""
        ), for: .normal)
        return button
    }()

    let cancelButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString(
            "REDEEM_VOUCHER_CANCEL_BUTTON",
            tableName: "RedeemVoucher",
            value: "Cancel",
            comment: ""
        ), for: .normal)
        return button
    }()

    private lazy var statusStack: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [activityIndicator, statusLabel])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .horizontal
        stackView.spacing = 8
        return stackView
    }()

    private lazy var topStackView: UIStackView = {
        let stackView =
            UIStackView(arrangedSubviews: [
                titleLabel,
                instructionLabel,
                inputTextField,
                statusStack,
            ])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = 8
        return stackView
    }()

    private lazy var bottomStackView: UIStackView = {
        let stackView = UIStackView(
            arrangedSubviews: [redeemButton, cancelButton]
        )
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.sectionSpacing
        return stackView
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        translatesAutoresizingMaskIntoConstraints = false

        backgroundColor = .secondaryColor

        setUpSubviews()

        layoutMargins = UIMetrics.contentLayoutMargins
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

private extension RedeemVoucherContentView {
    func setUpSubviews() {
        addSubview(topStackView)
        addSubview(bottomStackView)
        configureConstraints()
    }

    func configureConstraints() {
        NSLayoutConstraint.activate([
            topStackView.centerYAnchor.constraint(
                equalTo: centerYAnchor,
                constant: UIMetrics.verticalCenterOffset
            ),

            topStackView.leadingAnchor.constraint(
                equalTo: layoutMarginsGuide.leadingAnchor
            ),

            topStackView.trailingAnchor.constraint(
                equalTo: layoutMarginsGuide.trailingAnchor
            ),

            bottomStackView.leadingAnchor.constraint(
                equalTo: layoutMarginsGuide.leadingAnchor
            ),

            bottomStackView.trailingAnchor.constraint(
                equalTo: layoutMarginsGuide.trailingAnchor
            ),

            bottomStackView.bottomAnchor.constraint(
                equalTo: layoutMarginsGuide.bottomAnchor
            ),
        ])
    }
}
