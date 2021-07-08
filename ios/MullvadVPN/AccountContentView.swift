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

    let logoutButton: AppButton = {
        let button = AppButton(style: .danger)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString(
            "LOGOUT_BUTTON_TITLE",
            tableName: "Account",
            value: "Log out",
            comment: ""
        ), for: .normal)
        return button
    }()

    let accountTokenRowView: AccountTokenRow = {
        let view = AccountTokenRow()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    let accountExpiryRowView: AccountExpiryRow = {
        let view = AccountExpiryRow()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    lazy var contentStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [accountTokenRowView, accountExpiryRowView])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.sectionSpacing
        return stackView
    }()

    lazy var buttonStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [purchaseButton, restorePurchasesButton, logoutButton])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.interButtonSpacing
        stackView.setCustomSpacing(UIMetrics.interButtonSpacing, after: restorePurchasesButton)
        return stackView
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        layoutMargins = UIMetrics.contentLayoutMargins

        addSubview(contentStackView)
        addSubview(buttonStackView)

        NSLayoutConstraint.activate([
            contentStackView.topAnchor.constraint(equalTo: layoutMarginsGuide.topAnchor),
            contentStackView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            contentStackView.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),

            buttonStackView.topAnchor.constraint(greaterThanOrEqualTo: contentStackView.bottomAnchor, constant: UIMetrics.sectionSpacing),
            buttonStackView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            buttonStackView.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            buttonStackView.bottomAnchor.constraint(equalTo: layoutMarginsGuide.bottomAnchor)
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

class AccountTokenRow: UIView {

    var value: String? {
        didSet {
            valueButton.setTitle(value, for: .normal)
            accessibilityValue = value
        }
    }
    var actionHandler: (() -> Void)?

    private let textLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.text = NSLocalizedString("ACCOUNT_TOKEN_LABEL", tableName: "Account", comment: "")
        textLabel.font = UIFont.systemFont(ofSize: 14)
        textLabel.textColor = UIColor(white: 1.0, alpha: 0.6)
        return textLabel
    }()

    private let valueButton: UIButton = {
        let button = UIButton(type: .system)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.titleLabel?.font = UIFont.systemFont(ofSize: 17)
        button.setTitleColor(.white, for: .normal)
        button.contentHorizontalAlignment = .leading
        button.contentEdgeInsets = UIEdgeInsets(top: 0, left: 0, bottom: 0, right: 1)
        button.accessibilityHint = NSLocalizedString("ACCOUNT_TOKEN_ACCESSIBILITY_HINT", tableName: "Account", comment: "")
        return button
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        addSubview(textLabel)
        addSubview(valueButton)

        NSLayoutConstraint.activate([
            textLabel.topAnchor.constraint(equalTo: topAnchor),
            textLabel.leadingAnchor.constraint(equalTo: leadingAnchor),
            textLabel.trailingAnchor.constraint(greaterThanOrEqualTo: trailingAnchor),

            valueButton.topAnchor.constraint(equalTo: textLabel.bottomAnchor, constant: 8),
            valueButton.leadingAnchor.constraint(equalTo: leadingAnchor),
            valueButton.trailingAnchor.constraint(equalTo: trailingAnchor),
            valueButton.bottomAnchor.constraint(equalTo: bottomAnchor)
        ])

        isAccessibilityElement = true
        accessibilityLabel = textLabel.text

        let actionName = NSLocalizedString(
            "ACCOUNT_TOKEN_ACCESSIBILITY_ACTION_TITLE",
            tableName: "Account",
            comment: ""
        )
        accessibilityCustomActions = [UIAccessibilityCustomAction(name: actionName, target: self, selector: #selector(performAccessibilityAction))]

        valueButton.addTarget(self, action: #selector(handleTap), for: .touchUpInside)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    @objc private func handleTap() {
        self.actionHandler?()
    }

    @objc private func performAccessibilityAction() {
        self.actionHandler?()
    }
}

class AccountExpiryRow: UIView {

    var value: Date? {
        didSet {
            let expiry = value.flatMap { AccountExpiry(date: $0) }

            if let expiry = expiry, expiry.isExpired  {
                let localizedString = NSLocalizedString(
                    "ACCOUNT_OUT_OF_TIME_LABEL",
                    tableName: "Account",
                    value: "OUT OF TIME",
                    comment: "Label displayed in place of account expiration when account is out of time."
                )

                valueLabel.text = localizedString
                accessibilityValue = localizedString

                valueLabel.textColor = .dangerColor
            } else {
                let formattedDate = expiry?.formattedDate

                valueLabel.text = formattedDate
                accessibilityValue = formattedDate

                valueLabel.textColor = .white
            }
        }
    }

    private let textLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.text = NSLocalizedString("ACCOUNT_EXPIRY_LABEL", tableName: "Account", comment: "")
        textLabel.font = UIFont.systemFont(ofSize: 14)
        textLabel.textColor = UIColor(white: 1.0, alpha: 0.6)
        return textLabel
    }()

    private let valueLabel: UILabel = {
        let valueLabel = UILabel()
        valueLabel.translatesAutoresizingMaskIntoConstraints = false
        valueLabel.font = UIFont.systemFont(ofSize: 17)
        valueLabel.textColor = .white
        return valueLabel
    }()

    let activityIndicator: SpinnerActivityIndicatorView = {
        let activityIndicator = SpinnerActivityIndicatorView(style: .small)
        activityIndicator.translatesAutoresizingMaskIntoConstraints = false
        activityIndicator.tintColor = .white
        activityIndicator.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        activityIndicator.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        return activityIndicator
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        addSubview(textLabel)
        addSubview(activityIndicator)
        addSubview(valueLabel)

        NSLayoutConstraint.activate([
            textLabel.topAnchor.constraint(equalTo: topAnchor),
            textLabel.leadingAnchor.constraint(equalTo: leadingAnchor),
            textLabel.trailingAnchor.constraint(greaterThanOrEqualTo: activityIndicator.leadingAnchor, constant: -8),

            activityIndicator.topAnchor.constraint(equalTo: textLabel.topAnchor),
            activityIndicator.bottomAnchor.constraint(equalTo: textLabel.bottomAnchor),
            activityIndicator.trailingAnchor.constraint(equalTo: trailingAnchor),

            valueLabel.topAnchor.constraint(equalTo: textLabel.bottomAnchor, constant: 8),
            valueLabel.leadingAnchor.constraint(equalTo: leadingAnchor),
            valueLabel.trailingAnchor.constraint(equalTo: trailingAnchor),
            valueLabel.bottomAnchor.constraint(equalTo: bottomAnchor)
        ])

        isAccessibilityElement = true
        accessibilityLabel = textLabel.text
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
