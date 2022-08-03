//
//  WireguardKeysContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 05/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class WireguardKeysContentView: UIView {
    let regenerateKeyButton: AppButton = {
        let button = AppButton(style: .success)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(
            NSLocalizedString(
                "REGENERATE_KEY_BUTTON_TITLE",
                tableName: "WireguardKeys",
                value: "Regenerate key",
                comment: ""
            ),
            for: .normal
        )
        return button
    }()

    let verifyKeyButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(
            NSLocalizedString(
                "VERIFY_KEY_BUTTON_TITLE",
                tableName: "WireguardKeys",
                value: "Verify key",
                comment: ""
            ),
            for: .normal
        )
        return button
    }()

    let publicKeyRowView: WireguardKeysPublicKeyRow = {
        let view = WireguardKeysPublicKeyRow()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    let creationRowView: WireguardKeysCreationRow = {
        let view = WireguardKeysCreationRow()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    lazy var contentStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [publicKeyRowView, creationRowView])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.sectionSpacing
        return stackView
    }()

    lazy var buttonStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [regenerateKeyButton, verifyKeyButton])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.interButtonSpacing
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

class WireguardKeysPublicKeyRow: UIView {
    var value: String? {
        didSet {
            valueButton.setTitle(value, for: .normal)
            accessibilityValue = value
        }
    }

    var status: WireguardKeyStatusView.Status = .default {
        didSet {
            statusView.status = status
            updateAccessibilityLabel()
        }
    }

    var actionHandler: (() -> Void)?

    private let textLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.text = NSLocalizedString(
            "PUBLIC_KEY_LABEL",
            tableName: "WireguardKeys",
            value: "Public key",
            comment: ""
        )
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
        button.accessibilityHint = NSLocalizedString(
            "PUBLIC_KEY_ACCESSIBILITY_HINT",
            tableName: "WireguardKeys",
            value: "Tap to copy to pasteboard.",
            comment: ""
        )
        return button
    }()

    private let statusView: WireguardKeyStatusView = {
        let view = WireguardKeyStatusView()
        view.translatesAutoresizingMaskIntoConstraints = false
        view.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        view.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        return view
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        [textLabel, valueButton, statusView].forEach { subview in
            addSubview(subview)
        }

        NSLayoutConstraint.activate([
            textLabel.topAnchor.constraint(equalTo: topAnchor),
            textLabel.leadingAnchor.constraint(equalTo: leadingAnchor),
            textLabel.trailingAnchor.constraint(
                greaterThanOrEqualTo: statusView.leadingAnchor,
                constant: -8
            ),

            statusView.topAnchor.constraint(equalTo: textLabel.topAnchor),
            statusView.bottomAnchor.constraint(equalTo: textLabel.bottomAnchor),
            statusView.trailingAnchor.constraint(equalTo: trailingAnchor),

            valueButton.topAnchor.constraint(equalTo: textLabel.bottomAnchor, constant: 8),
            valueButton.leadingAnchor.constraint(equalTo: leadingAnchor),
            valueButton.trailingAnchor.constraint(equalTo: trailingAnchor),
            valueButton.bottomAnchor.constraint(equalTo: bottomAnchor),
        ])

        isAccessibilityElement = true
        updateAccessibilityLabel()

        let actionName = NSLocalizedString(
            "ACCOUNT_TOKEN_ACCESSIBILITY_ACTION_TITLE",
            tableName: "WireguardKeys",
            value: "Copy account token to pasteboard",
            comment: ""
        )
        accessibilityCustomActions = [
            UIAccessibilityCustomAction(
                name: actionName,
                target: self,
                selector: #selector(performAccessibilityAction)
            ),
        ]

        valueButton.addTarget(self, action: #selector(handleTap), for: .touchUpInside)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func updateAccessibilityLabel() {
        var accessibilityLabelString = textLabel.text ?? ""

        if case let .verified(isValid) = status {
            accessibilityLabelString += ", "

            if isValid {
                accessibilityLabelString.append(
                    NSLocalizedString(
                        "KEY_STATUS_VALID",
                        tableName: "WireguardKeys",
                        value: "Key is valid",
                        comment: ""
                    )
                )
            } else {
                accessibilityLabelString.append(
                    NSLocalizedString(
                        "KEY_STATUS_INVALID",
                        tableName: "WireguardKeys",
                        value: "Key is invalid",
                        comment: ""
                    )
                )
            }
        }

        accessibilityLabel = accessibilityLabelString
    }

    @objc private func handleTap() {
        actionHandler?()
    }

    @objc private func performAccessibilityAction() {
        actionHandler?()
    }
}

class WireguardKeysCreationRow: UIView {
    var value: String? {
        didSet {
            accessibilityValue = value
            valueLabel.text = value
        }
    }

    private let textLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.text = NSLocalizedString(
            "KEY_GENERATED_LABEL",
            tableName: "WireguardKeys",
            value: "Key generated",
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

    override init(frame: CGRect) {
        super.init(frame: frame)

        addSubview(textLabel)
        addSubview(valueLabel)

        NSLayoutConstraint.activate([
            textLabel.topAnchor.constraint(equalTo: topAnchor),
            textLabel.leadingAnchor.constraint(equalTo: leadingAnchor),
            textLabel.trailingAnchor.constraint(equalTo: trailingAnchor),

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

class WireguardKeyStatusView: UIView {
    enum Status {
        case `default`, verifying, verified(Bool), regenerating
    }

    let textLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.font = UIFont.systemFont(ofSize: 14)
        textLabel.textColor = .successColor
        return textLabel
    }()

    let activityIndicator: SpinnerActivityIndicatorView = {
        let activityIndicator = SpinnerActivityIndicatorView(style: .small)
        activityIndicator.translatesAutoresizingMaskIntoConstraints = false
        activityIndicator.tintColor = .white
        return activityIndicator
    }()

    lazy var stackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [textLabel, activityIndicator])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.spacing = 4
        stackView.axis = .horizontal
        stackView.distribution = .equalCentering
        return stackView
    }()

    var status: Status = .default {
        didSet {
            updateView()
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)

        addSubview(stackView)

        NSLayoutConstraint.activate([
            stackView.topAnchor.constraint(equalTo: topAnchor),
            stackView.leadingAnchor.constraint(equalTo: leadingAnchor),
            stackView.bottomAnchor.constraint(equalTo: bottomAnchor),
            stackView.trailingAnchor.constraint(equalTo: trailingAnchor),
        ])

        updateView()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func updateView() {
        switch status {
        case .default:
            textLabel.isHidden = true
            activityIndicator.stopAnimating()

        case .regenerating, .verifying:
            startSpinner()

        case let .verified(isValid):
            textLabel.isHidden = false
            activityIndicator.stopAnimating()

            if isValid {
                textLabel.text = NSLocalizedString(
                    "KEY_STATUS_VALID",
                    tableName: "WireguardKeys",
                    value: "Key is valid",
                    comment: ""
                )
                textLabel.textColor = .successColor
            } else {
                textLabel.text = NSLocalizedString(
                    "KEY_STATUS_INVALID",
                    tableName: "WireguardKeys",
                    value: "Key is invalid",
                    comment: ""
                )
                textLabel.textColor = .dangerColor
            }
        }
    }

    private func startSpinner() {
        textLabel.isHidden = true
        activityIndicator.startAnimating()
    }
}
