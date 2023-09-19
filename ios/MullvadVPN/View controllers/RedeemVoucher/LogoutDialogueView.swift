//
//  LogoutDialogueView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-29.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class LogoutDialogueView: UIView {
    private let containerView: UIView = {
        let view = UIView()
        view.backgroundColor = .secondaryColor
        view.layer.cornerRadius = 11
        view.directionalLayoutMargins = UIMetrics.CustomAlert.containerMargins
        view.clipsToBounds = true
        return view
    }()

    private let messageLabel: UILabel = {
        let label = UILabel()
        label.font = .preferredFont(forTextStyle: .callout, weight: .semibold)
        label.numberOfLines = .zero
        label.lineBreakMode = .byWordWrapping
        label.textColor = .white
        label.text = NSLocalizedString(
            "ACCOUNT_NUMBER_AS_VOUCHER_INPUT_ERROR_BODY",
            tableName: "CreateAccountRedeemingVoucher",
            value: """
            It looks like you have entered a Mullvad account number instead of a voucher code. \
            Do you want to log in to an existing account?
            If so, click log out below to log in with the other account number.
            """,
            comment: ""
        )
        return label
    }()

    private let logoutButton: AppButton = {
        let button = AppButton(style: .danger)
        button.setTitle(NSLocalizedString(
            "LOGOUT_BUTTON_TITLE",
            tableName: "CreateAccountRedeemingVoucher",
            value: "Log out",
            comment: ""
        ), for: .normal)
        return button
    }()

    private var showConstraint: NSLayoutConstraint?
    private var hideConstraint: NSLayoutConstraint?
    private var didRequestToLogOut: (LogoutDialogueView) -> Void

    var isLoading = true {
        didSet {
            logoutButton.isEnabled = !isLoading
        }
    }

    override var isHidden: Bool {
        willSet {
            if newValue == true {
                fadeOut()
            } else {
                fadeIn()
            }
        }
    }

    init(didRequestToLogOut: @escaping (LogoutDialogueView) -> Void) {
        self.didRequestToLogOut = didRequestToLogOut
        super.init(frame: .zero)
        setupAppearance()
        configureUI()
        addActions()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupAppearance() {
        containerView.layer.cornerRadius = 11
        containerView.backgroundColor = .primaryColor
    }

    private func configureUI() {
        addConstrainedSubviews([containerView]) {
            containerView.pinEdgesToSuperview(.all().excluding(.bottom))
        }

        containerView.addConstrainedSubviews([messageLabel, logoutButton]) {
            messageLabel.pinEdgesToSuperviewMargins(.all().excluding(.bottom))
            logoutButton.pinEdgesToSuperviewMargins(.all().excluding(.top))
            logoutButton.topAnchor.constraint(
                equalTo: messageLabel.bottomAnchor,
                constant: UIMetrics.padding16
            ).withPriority(.defaultHigh)
        }

        showConstraint = containerView.bottomAnchor.constraint(equalTo: bottomAnchor)
        hideConstraint = containerView.bottomAnchor.constraint(equalTo: topAnchor)
        hideConstraint?.isActive = true
    }

    private func addActions() {
        logoutButton.addTarget(self, action: #selector(logout), for: .touchUpInside)
    }

    @objc private func logout() {
        didRequestToLogOut(self)
    }

    private func fadeIn() {
        guard hideConstraint?.isActive == true else { return }
        showConstraint?.isActive = true
        hideConstraint?.isActive = false
        animateWith(animations: {
            self.containerView.alpha = 1.0
        }, duration: 0.3, delay: 0.2)
    }

    private func fadeOut() {
        guard showConstraint?.isActive == true else { return }
        showConstraint?.isActive = false
        hideConstraint?.isActive = true
        animateWith(animations: {
            self.containerView.alpha = 0.0
        }, duration: 0.0, delay: 0.0)
    }

    private func animateWith(
        animations: @escaping () -> Void,
        duration: TimeInterval,
        delay: TimeInterval
    ) {
        UIView.animate(
            withDuration: duration,
            delay: delay,
            options: .curveEaseInOut,
            animations: {
                animations()
                self.layoutIfNeeded()
            },
            completion: nil
        )
    }
}
