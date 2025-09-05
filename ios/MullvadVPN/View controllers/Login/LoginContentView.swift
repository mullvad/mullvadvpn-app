//
//  LoginContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 22/03/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class LoginContentView: UIView {
    private var keyboardResponder: AutomaticKeyboardResponder?

    let titleLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = .mullvadBig
        textLabel.textColor = .white
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.adjustsFontForContentSizeCategory = true
        return textLabel
    }()

    let messageLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = .mullvadTinySemiBold
        textLabel.textColor = UIColor.white.withAlphaComponent(0.6)
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.adjustsFontForContentSizeCategory = true
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
        statusActivityView.clipsToBounds = true
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
        textLabel.font = .mullvadSmall
        textLabel.textColor = UIColor.white.withAlphaComponent(0.6)
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.adjustsFontForContentSizeCategory = true
        textLabel.numberOfLines = 0
        textLabel.text = NSLocalizedString("Don’t have an account number?", comment: "")
        textLabel.numberOfLines = 0
        return textLabel
    }()

    let createAccountButton: AppButton = {
        let button = AppButton(style: .default)
        button.setAccessibilityIdentifier(.createAccountButton)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString("Create account", comment: ""), for: .normal)
        return button
    }()

    private let scrollView = UIScrollView()

    private var isStatusImageVisible = false
    private var contentContainerBottomConstraint: NSLayoutConstraint?

    override init(frame: CGRect) {
        super.init(frame: frame)

        backgroundColor = .primaryColor
        directionalLayoutMargins = UIMetrics.contentLayoutMargins
        setAccessibilityIdentifier(.loginView)

        accountInputGroup.textField.setAccessibilityIdentifier(.loginTextField)

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

        scrollView.addConstrainedSubviews([contentContainer]) {
            contentContainer.widthAnchor.constraint(equalTo: scrollView.widthAnchor)
            contentContainer.topAnchor.constraint(equalTo: scrollView.contentLayoutGuide.topAnchor)
            contentContainer.bottomAnchor.constraint(equalTo: scrollView.contentLayoutGuide.bottomAnchor)

            statusActivityView.topAnchor.constraint(greaterThanOrEqualTo: contentContainer.topAnchor)
            statusActivityView.centerXAnchor.constraint(equalTo: contentContainer.centerXAnchor)
            statusActivityView.widthAnchor.constraint(equalToConstant: 60.0)
            statusActivityView.heightAnchor.constraint(equalTo: statusActivityView.widthAnchor, multiplier: 1.0)

            formContainer.topAnchor.constraint(equalTo: statusActivityView.bottomAnchor, constant: 30)
            formContainer.centerYAnchor.constraint(equalTo: contentContainer.centerYAnchor, constant: -20)
            formContainer.bottomAnchor.constraint(lessThanOrEqualTo: contentContainer.bottomAnchor)
            formContainer.pinEdgesToSuperview(PinnableEdges([.leading(0), .trailing(0)]))
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

        let contentContainerBottomConstraint = bottomAnchor
            .constraint(equalTo: scrollView.bottomAnchor)
        self.contentContainerBottomConstraint = contentContainerBottomConstraint

        addConstrainedSubviews([scrollView, footerContainer]) {
            scrollView.pinEdges(PinnableEdges([.top(0)]), to: safeAreaLayoutGuide)
            scrollView.pinEdgesToSuperview(.all().excluding(.top))

            footerContainer.pinEdgesToSuperview(.all().excluding(.top))
            footerLabel.pinEdges(.all().excluding(.bottom), to: footerContainer.layoutMarginsGuide)

            createAccountButton.topAnchor.constraint(equalToSystemSpacingBelow: footerLabel.bottomAnchor, multiplier: 1)
            createAccountButton.pinEdges(.all().excluding(.top), to: footerContainer.layoutMarginsGuide)
        }
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        updateScrollViewContentInset()
    }

    private func updateScrollViewContentInset() {
        scrollView.contentInset = .init(
            top: 0,
            left: 0,
            bottom: footerContainer.frame.height,
            right: 0
        )
    }
}
