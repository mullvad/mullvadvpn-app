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
        view.directionalLayoutMargins = UIMetrics.contentLayoutMargins
        return view
    }()

    let formContainer: UIView = {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        view.directionalLayoutMargins = UIMetrics.contentLayoutMargins
        return view
    }()

    let footerContainer: UIView = {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        view.directionalLayoutMargins = UIMetrics.contentLayoutMargins
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
            value: "Don’t have an account number?",
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
        directionalLayoutMargins = UIMetrics.contentLayoutMargins

        accountInputGroup.textField.accessibilityIdentifier = AccessibilityIdentifier.loginTextField.rawValue

        keyboardResponder = AutomaticKeyboardResponder(
            targetView: self,
            handler: { [weak self] _, adjustment in
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

        let contentContainerBottomConstraint = bottomAnchor
            .constraint(equalTo: contentContainer.bottomAnchor)
        self.contentContainerBottomConstraint = contentContainerBottomConstraint

        addConstrainedSubviews([contentContainer, footerContainer]) {
            contentContainer.pinEdges(PinnableEdges([.top(0)]), to: safeAreaLayoutGuide)
            contentContainer.pinEdgesToSuperview(PinnableEdges([.leading(0), .trailing(0)]))
            contentContainerBottomConstraint

            footerContainer.pinEdgesToSuperview(.all().excluding(.top))
            footerLabel.pinEdges(.all().excluding(.bottom), to: footerContainer.layoutMarginsGuide)

            createAccountButton.topAnchor.constraint(equalToSystemSpacingBelow: footerLabel.bottomAnchor, multiplier: 1)
            createAccountButton.pinEdges(.all().excluding(.top), to: footerContainer.layoutMarginsGuide)

            statusActivityView.centerXAnchor.constraint(equalTo: contentContainer.centerXAnchor)

            formContainer.topAnchor.constraint(equalTo: statusActivityView.bottomAnchor, constant: 30)
            formContainer.centerYAnchor.constraint(equalTo: contentContainer.centerYAnchor, constant: -20)
            formContainer.pinEdges(PinnableEdges([.leading(0), .trailing(0)]), to: contentContainer)
            formContainer.pinEdges(PinnableEdges([.bottom(0)]), to: accountInputGroupWrapper)

            titleLabel.pinEdges(.all().excluding(.bottom), to: formContainer.layoutMarginsGuide)

            messageLabel.topAnchor.constraint(equalToSystemSpacingBelow: titleLabel.bottomAnchor, multiplier: 1)
            messageLabel.pinEdges(PinnableEdges([.leading(0), .trailing(0)]), to: formContainer.layoutMarginsGuide)

            accountInputGroupWrapper.topAnchor.constraint(
                equalToSystemSpacingBelow: messageLabel.bottomAnchor,
                multiplier: 1
            )
            accountInputGroupWrapper.pinEdges(
                PinnableEdges([.leading(0), .trailing(0)]),
                to: formContainer.layoutMarginsGuide
            )
            accountInputGroupWrapper.heightAnchor.constraint(equalTo: accountInputGroup.contentView.heightAnchor)
            accountInputGroup.pinEdges(.all().excluding(.bottom), to: accountInputGroupWrapper)
        }
    }
}
