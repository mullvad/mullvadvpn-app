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
        stackView.spacing = UIMetrics.sectionSpacing
        return stackView
    }()

    lazy var buttonStackView: UIStackView = {
        let stackView =
            UIStackView(arrangedSubviews: [purchaseButton, restorePurchasesButton, logoutButton])
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

            buttonStackView.topAnchor.constraint(
                greaterThanOrEqualTo: contentStackView.bottomAnchor,
                constant: UIMetrics.sectionSpacing
            ),
            buttonStackView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            buttonStackView.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            buttonStackView.bottomAnchor.constraint(equalTo: layoutMarginsGuide.bottomAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

class AccountDeviceRow: UIView {
    var deviceName: String? {
        didSet {
            deviceLabel.text = deviceName?.capitalized ?? ""
            accessibilityValue = deviceName
        }
    }

    private let titleLabel: UILabel = {
        let label = UILabel()
        label.translatesAutoresizingMaskIntoConstraints = false
        label.text = NSLocalizedString(
            "DEVICE_NAME",
            tableName: "Account",
            value: "Device name",
            comment: ""
        )
        label.font = UIFont.systemFont(ofSize: 14)
        label.textColor = UIColor(white: 1.0, alpha: 0.6)
        return label
    }()

    private let deviceLabel: UILabel = {
        let label = UILabel()
        label.translatesAutoresizingMaskIntoConstraints = false
        label.font = UIFont.systemFont(ofSize: 17)
        label.textColor = .white
        return label
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        addSubview(titleLabel)
        addSubview(deviceLabel)

        NSLayoutConstraint.activate([
            titleLabel.topAnchor.constraint(equalTo: topAnchor),
            titleLabel.leadingAnchor.constraint(equalTo: leadingAnchor),
            titleLabel.trailingAnchor.constraint(equalTo: trailingAnchor),

            deviceLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 8),
            deviceLabel.leadingAnchor.constraint(equalTo: leadingAnchor),
            deviceLabel.trailingAnchor.constraint(equalTo: trailingAnchor),
            deviceLabel.bottomAnchor.constraint(equalTo: bottomAnchor),
        ])

        isAccessibilityElement = true
        accessibilityLabel = titleLabel.text
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

class AccountNumberRow: UIView {
    var accountNumber: String? {
        didSet {
            updateView()
        }
    }

    var isObscured = true {
        didSet {
            updateView()
        }
    }

    var copyAccountNumber: (() -> Void)?

    private let titleLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.text = NSLocalizedString(
            "ACCOUNT_TOKEN_LABEL",
            tableName: "Account",
            value: "Account number",
            comment: ""
        )
        textLabel.font = UIFont.systemFont(ofSize: 14)
        textLabel.textColor = UIColor(white: 1.0, alpha: 0.6)
        return textLabel
    }()

    private let accountNumberLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.font = UIFont.monospacedSystemFont(ofSize: 17, weight: .regular)
        textLabel.textColor = .white
        return textLabel
    }()

    private let showHideButton: UIButton = {
        let button = UIButton(type: .system)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.tintColor = .white
        button.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        return button
    }()

    private let copyButton: UIButton = {
        let button = UIButton(type: .system)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.tintColor = .white
        button.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        return button
    }()

    private var revertCopyImageWorkItem: DispatchWorkItem?

    override init(frame: CGRect) {
        super.init(frame: frame)

        addSubview(titleLabel)
        addSubview(accountNumberLabel)
        addSubview(showHideButton)
        addSubview(copyButton)

        NSLayoutConstraint.activate([
            titleLabel.topAnchor.constraint(equalTo: topAnchor),
            titleLabel.leadingAnchor.constraint(equalTo: leadingAnchor),
            titleLabel.trailingAnchor.constraint(greaterThanOrEqualTo: trailingAnchor),

            accountNumberLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 8),
            accountNumberLabel.leadingAnchor.constraint(equalTo: leadingAnchor),
            accountNumberLabel.trailingAnchor.constraint(equalTo: showHideButton.leadingAnchor),
            accountNumberLabel.bottomAnchor.constraint(equalTo: bottomAnchor),

            showHideButton.heightAnchor.constraint(equalTo: accountNumberLabel.heightAnchor),
            showHideButton.centerYAnchor.constraint(equalTo: accountNumberLabel.centerYAnchor),
            showHideButton.leadingAnchor.constraint(equalTo: accountNumberLabel.trailingAnchor),

            copyButton.heightAnchor.constraint(equalTo: accountNumberLabel.heightAnchor),
            copyButton.centerYAnchor.constraint(equalTo: accountNumberLabel.centerYAnchor),
            copyButton.leadingAnchor.constraint(
                equalTo: showHideButton.trailingAnchor,
                constant: 24
            ),
            copyButton.trailingAnchor.constraint(equalTo: trailingAnchor),
        ])

        showHideButton.addTarget(
            self,
            action: #selector(didTapShowHideAccount),
            for: .touchUpInside
        )

        copyButton.addTarget(
            self,
            action: #selector(didTapCopyAccountNumber),
            for: .touchUpInside
        )

        isAccessibilityElement = true
        accessibilityLabel = titleLabel.text

        showCheckmark(false)
        updateView()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - Private

    private func updateView() {
        accountNumberLabel.text = displayAccountNumber ?? ""
        showHideButton.setImage(showHideImage, for: .normal)

        accessibilityAttributedValue = _accessibilityAttributedValue
        accessibilityCustomActions = _accessibilityCustomActions
    }

    private var displayAccountNumber: String? {
        guard let accountNumber = accountNumber else {
            return nil
        }

        let formattedString = StringFormatter.formattedAccountNumber(from: accountNumber)

        if isObscured {
            return String(formattedString.map { ch in
                return ch == " " ? ch : "•"
            })
        } else {
            return formattedString
        }
    }

    private var showHideImage: UIImage? {
        if isObscured {
            return UIImage(named: "IconUnobscure")
        } else {
            return UIImage(named: "IconObscure")
        }
    }

    private var _accessibilityAttributedValue: NSAttributedString? {
        guard let accountNumber = accountNumber else {
            return nil
        }

        if isObscured {
            return NSAttributedString(
                string: NSLocalizedString(
                    "ACCOUNT_ACCESSIBILITY_OBSCURED",
                    tableName: "Account",
                    value: "Obscured",
                    comment: ""
                )
            )
        } else {
            return NSAttributedString(
                string: accountNumber,
                attributes: [.accessibilitySpeechSpellOut: true]
            )
        }
    }

    private var _accessibilityCustomActions: [UIAccessibilityCustomAction]? {
        guard accountNumber != nil else { return nil }

        return [
            UIAccessibilityCustomAction(
                name: showHideAccessibilityActionName,
                target: self,
                selector: #selector(didTapShowHideAccount)
            ),
            UIAccessibilityCustomAction(
                name: NSLocalizedString(
                    "ACCOUNT_ACCESSIBILITY_COPY_TO_PASTEBOARD",
                    tableName: "Account",
                    value: "Copy to pasteboard",
                    comment: ""
                ),
                target: self,
                selector: #selector(didTapCopyAccountNumber)
            ),
        ]
    }

    private var showHideAccessibilityActionName: String {
        if isObscured {
            return NSLocalizedString(
                "ACCOUNT_ACCESSIBILITY_SHOW_ACCOUNT_NUMBER",
                tableName: "Account",
                value: "Show account number",
                comment: ""
            )
        } else {
            return NSLocalizedString(
                "ACCOUNT_ACCESSIBILITY_HIDE_ACCOUNT_NUMBER",
                tableName: "Account",
                value: "Hide account number",
                comment: ""
            )
        }
    }

    private func showCheckmark(_ showCheckmark: Bool) {
        if showCheckmark {
            let tickIcon = UIImage(named: "IconTick")

            copyButton.setImage(tickIcon, for: .normal)
            copyButton.tintColor = .successColor
        } else {
            let copyIcon = UIImage(named: "IconCopy")

            copyButton.setImage(copyIcon, for: .normal)
            copyButton.tintColor = .white
        }
    }

    // MARK: - Actions

    @objc private func didTapShowHideAccount() {
        isObscured.toggle()
        updateView()

        UIAccessibility.post(notification: .layoutChanged, argument: nil)
    }

    @objc private func didTapCopyAccountNumber() {
        let delayedWorkItem = DispatchWorkItem { [weak self] in
            self?.showCheckmark(false)
        }

        revertCopyImageWorkItem?.cancel()
        revertCopyImageWorkItem = delayedWorkItem

        showCheckmark(true)
        copyAccountNumber?()

        DispatchQueue.main.asyncAfter(
            deadline: .now() + .seconds(2),
            execute: delayedWorkItem
        )
    }
}

class AccountExpiryRow: UIView {
    var value: Date? {
        didSet {
            let expiry = value

            if let expiry = expiry, expiry <= Date() {
                let localizedString = NSLocalizedString(
                    "ACCOUNT_OUT_OF_TIME_LABEL",
                    tableName: "Account",
                    value: "OUT OF TIME",
                    comment: ""
                )

                valueLabel.text = localizedString
                accessibilityValue = localizedString

                valueLabel.textColor = .dangerColor
            } else {
                let formattedDate = expiry.map { date in
                    return DateFormatter.localizedString(
                        from: date,
                        dateStyle: .medium,
                        timeStyle: .short
                    )
                }

                valueLabel.text = formattedDate ?? ""
                accessibilityValue = formattedDate

                valueLabel.textColor = .white
            }
        }
    }

    private let textLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.text = NSLocalizedString(
            "ACCOUNT_EXPIRY_LABEL",
            tableName: "Account",
            value: "Paid until",
            comment: ""
        )
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
            textLabel.trailingAnchor.constraint(
                greaterThanOrEqualTo: activityIndicator.leadingAnchor,
                constant: -8
            ),

            activityIndicator.topAnchor.constraint(equalTo: textLabel.topAnchor),
            activityIndicator.bottomAnchor.constraint(equalTo: textLabel.bottomAnchor),
            activityIndicator.trailingAnchor.constraint(equalTo: trailingAnchor),

            valueLabel.topAnchor.constraint(equalTo: textLabel.bottomAnchor, constant: 8),
            valueLabel.leadingAnchor.constraint(equalTo: leadingAnchor),
            valueLabel.trailingAnchor.constraint(equalTo: trailingAnchor),
            valueLabel.bottomAnchor.constraint(equalTo: bottomAnchor),
        ])

        isAccessibilityElement = true
        accessibilityLabel = textLabel.text
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
