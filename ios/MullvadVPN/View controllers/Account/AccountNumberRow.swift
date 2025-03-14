//
//  AccountNumberRow.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-28.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

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
        textLabel.font = UIFont.monospacedSystemFont(ofSize: 17, weight: .regular)
        textLabel.textColor = .white
        return textLabel
    }()

    private let showHideButton: UIButton = {
        let button = UIButton(type: .system)
        button.tintColor = .white
        button.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        return button
    }()

    private let copyButton: UIButton = {
        let button = UIButton(type: .system)
        button.tintColor = .white
        button.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        return button
    }()

    private var revertCopyImageWorkItem: DispatchWorkItem?

    override init(frame: CGRect) {
        super.init(frame: frame)

        addConstrainedSubviews([titleLabel, accountNumberLabel, showHideButton, copyButton]) {
            titleLabel.pinEdgesToSuperview(.all().excluding([.trailing, .bottom]))
            titleLabel.trailingAnchor.constraint(greaterThanOrEqualTo: trailingAnchor)

            accountNumberLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: UIMetrics.padding8)
            accountNumberLabel.leadingAnchor.constraint(equalTo: leadingAnchor)
            accountNumberLabel.trailingAnchor.constraint(equalTo: showHideButton.leadingAnchor)
            accountNumberLabel.bottomAnchor.constraint(equalTo: bottomAnchor)

            showHideButton.heightAnchor.constraint(equalTo: accountNumberLabel.heightAnchor)
            showHideButton.centerYAnchor.constraint(equalTo: accountNumberLabel.centerYAnchor)
            showHideButton.leadingAnchor.constraint(equalTo: accountNumberLabel.trailingAnchor)

            copyButton.heightAnchor.constraint(equalTo: accountNumberLabel.heightAnchor)
            copyButton.centerYAnchor.constraint(equalTo: accountNumberLabel.centerYAnchor)
            copyButton.leadingAnchor.constraint(
                equalTo: showHideButton.trailingAnchor,
                constant: UIMetrics.padding24
            )
            copyButton.trailingAnchor.constraint(equalTo: trailingAnchor)
        }

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

    func setButtons(enabled: Bool) {
        showHideButton.isEnabled = enabled
        copyButton.isEnabled = enabled
    }

    // MARK: - Private

    private func updateView() {
        accountNumberLabel.text = displayAccountNumber ?? ""
        showHideButton.setImage(showHideImage, for: .normal)

        accessibilityAttributedValue = _accessibilityAttributedValue
        accessibilityCustomActions = _accessibilityCustomActions
    }

    private var displayAccountNumber: String? {
        guard let accountNumber else {
            return nil
        }

        let formattedString = accountNumber.formattedAccountNumber

        if isObscured {
            return String(formattedString.map { ch in
                ch == " " ? ch : "•"
            })
        } else {
            return formattedString
        }
    }

    private var showHideImage: UIImage? {
        if isObscured {
            return UIImage.Buttons.show
        } else {
            return UIImage.Buttons.hide
        }
    }

    private var _accessibilityAttributedValue: NSAttributedString? {
        guard let accountNumber else {
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
            let tickIcon = UIImage.tick

            copyButton.setImage(tickIcon, for: .normal)
            copyButton.tintColor = .successColor
        } else {
            let copyIcon = UIImage.Buttons.copy

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
