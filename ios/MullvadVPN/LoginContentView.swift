//
//  LoginContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 22/03/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class LoginContentView: UIView {
    private var keyboardResponder: AutomaticKeyboardResponder?

    let titleLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = UIFont.systemFont(ofSize: 32)
        textLabel.textColor = .white
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        return textLabel
    }()

    let messageLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = UIFont.systemFont(ofSize: 17)
        textLabel.textColor = UIColor.white.withAlphaComponent(0.6)
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.numberOfLines = 0
        return textLabel
    }()

    let accountInputGroup: AccountInputGroupView = {
        let inputGroup = AccountInputGroupView()
        inputGroup.translatesAutoresizingMaskIntoConstraints = false
        return inputGroup
    }()

    let accountInputGroupWrapper: UIView = {
        let wrapperView = UIView()
        wrapperView.translatesAutoresizingMaskIntoConstraints = false
        return wrapperView
    }()

    let statusActivityView: StatusActivityView = {
        let statusActivityView = StatusActivityView(state: .hidden)
        statusActivityView.translatesAutoresizingMaskIntoConstraints = false
        return statusActivityView
    }()

    let contentContainer: UIView = {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        view.layoutMargins = UIMetrics.contentLayoutMargins
        return view
    }()

    let formContainer: UIView = {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        view.layoutMargins = UIMetrics.contentLayoutMargins
        return view
    }()

    let footerContainer: UIView = {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        view.layoutMargins = UIMetrics.contentLayoutMargins
        view.backgroundColor = .secondaryColor
        return view
    }()

    let footerLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = UIFont.systemFont(ofSize: 17)
        textLabel.textColor = UIColor.white.withAlphaComponent(0.6)
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.text = NSLocalizedString(
            "CREATE_BUTTON_HEADER_LABEL",
            tableName: "Login",
            value: "Don't have an account number?",
            comment: ""
        )
        return textLabel
    }()

    let createAccountButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString(
            "CREATE_ACCOUNT_BUTTON_LABEL",
            tableName: "Login",
            value: "Create account",
            comment: ""
        ), for: .normal)
        return button
    }()

    private var isStatusImageVisible = false
    private var contentContainerBottomConstraint: NSLayoutConstraint?

    override init(frame: CGRect) {
        super.init(frame: frame)

        backgroundColor = .primaryColor
        layoutMargins = UIMetrics.contentLayoutMargins

        accountInputGroup.textField.accessibilityIdentifier = "LoginTextField"

        keyboardResponder = AutomaticKeyboardResponder(
            targetView: self,
            handler: { [weak self] view, adjustment in
                self?.contentContainerBottomConstraint?.constant = adjustment

                self?.layoutIfNeeded()
            }
        )

        addSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func addSubviews() {
        formContainer.addSubview(titleLabel)
        formContainer.addSubview(messageLabel)
        formContainer.addSubview(accountInputGroupWrapper)
        accountInputGroupWrapper.addSubview(accountInputGroup)

        contentContainer.addSubview(statusActivityView)
        contentContainer.addSubview(formContainer)

        footerContainer.addSubview(footerLabel)
        footerContainer.addSubview(createAccountButton)

        addSubview(contentContainer)
        addSubview(footerContainer)

        let contentContainerBottomConstraint = bottomAnchor
            .constraint(equalTo: contentContainer.bottomAnchor)
        self.contentContainerBottomConstraint = contentContainerBottomConstraint

        NSLayoutConstraint.activate([
            contentContainer.topAnchor.constraint(equalTo: safeAreaLayoutGuide.topAnchor),
            contentContainer.leadingAnchor.constraint(equalTo: leadingAnchor),
            contentContainer.trailingAnchor.constraint(equalTo: trailingAnchor),
            contentContainerBottomConstraint,

            footerContainer.leadingAnchor.constraint(equalTo: leadingAnchor),
            footerContainer.trailingAnchor.constraint(equalTo: trailingAnchor),
            footerContainer.bottomAnchor.constraint(equalTo: bottomAnchor),

            footerLabel.topAnchor.constraint(equalTo: footerContainer.layoutMarginsGuide.topAnchor),
            footerLabel.leadingAnchor
                .constraint(equalTo: footerContainer.layoutMarginsGuide.leadingAnchor),
            footerLabel.trailingAnchor
                .constraint(equalTo: footerContainer.layoutMarginsGuide.trailingAnchor),

            createAccountButton.topAnchor.constraint(
                equalToSystemSpacingBelow: footerLabel.bottomAnchor,
                multiplier: 1
            ),
            createAccountButton.leadingAnchor
                .constraint(equalTo: footerContainer.layoutMarginsGuide.leadingAnchor),
            createAccountButton.trailingAnchor
                .constraint(equalTo: footerContainer.layoutMarginsGuide.trailingAnchor),
            createAccountButton.bottomAnchor
                .constraint(equalTo: footerContainer.layoutMarginsGuide.bottomAnchor),

            statusActivityView.centerXAnchor.constraint(equalTo: contentContainer.centerXAnchor),
            formContainer.topAnchor.constraint(
                equalTo: statusActivityView.bottomAnchor,
                constant: 30
            ),
            formContainer.centerYAnchor.constraint(
                equalTo: contentContainer.centerYAnchor,
                constant: -20
            ),
            formContainer.leadingAnchor.constraint(equalTo: contentContainer.leadingAnchor),
            formContainer.trailingAnchor.constraint(equalTo: contentContainer.trailingAnchor),
            formContainer.bottomAnchor.constraint(equalTo: accountInputGroupWrapper.bottomAnchor),

            titleLabel.topAnchor.constraint(equalTo: formContainer.topAnchor),
            titleLabel.leadingAnchor
                .constraint(equalTo: formContainer.layoutMarginsGuide.leadingAnchor),
            titleLabel.trailingAnchor
                .constraint(equalTo: formContainer.layoutMarginsGuide.trailingAnchor),

            messageLabel.topAnchor.constraint(
                equalToSystemSpacingBelow: titleLabel.bottomAnchor,
                multiplier: 1
            ),
            messageLabel.leadingAnchor
                .constraint(equalTo: formContainer.layoutMarginsGuide.leadingAnchor),
            messageLabel.trailingAnchor
                .constraint(equalTo: formContainer.layoutMarginsGuide.trailingAnchor),

            accountInputGroupWrapper.topAnchor.constraint(
                equalToSystemSpacingBelow: messageLabel.bottomAnchor,
                multiplier: 1
            ),
            accountInputGroupWrapper.leadingAnchor
                .constraint(equalTo: formContainer.layoutMarginsGuide.leadingAnchor),
            accountInputGroupWrapper.trailingAnchor
                .constraint(equalTo: formContainer.layoutMarginsGuide.trailingAnchor),
            accountInputGroupWrapper.heightAnchor
                .constraint(equalTo: accountInputGroup.contentView.heightAnchor),

            accountInputGroup.topAnchor.constraint(equalTo: accountInputGroupWrapper.topAnchor),
            accountInputGroup.leadingAnchor
                .constraint(equalTo: accountInputGroupWrapper.leadingAnchor),
            accountInputGroup.trailingAnchor
                .constraint(equalTo: accountInputGroupWrapper.trailingAnchor),
        ])
    }
}
