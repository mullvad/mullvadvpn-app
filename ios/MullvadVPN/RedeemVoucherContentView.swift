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
    typealias Action = () -> Void

    // MARK: - Constants

    private let instructionLabelSuccessString = NSLocalizedString(
        "REDEEM_VOUCHER_INSTRUCTION_SUCCESS",
        tableName: "RedeemVoucher",
        value: "Voucher was successfully redeemed.",
        comment: ""
    )

    private let gotItButtonTitle = NSLocalizedString(
        "REDEEM_VOUCHER_GOT_IT_BUTTON",
        tableName: "RedeemVoucher",
        value: "Got it!",
        comment: ""
    )

    // MARK: - Views

    private let instructionLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 17)
        label.text = NSLocalizedString(
            "REDEEM_VOUCHER_INSTRUCTION",
            tableName: "RedeemVoucher",
            value: "Enter voucher code",
            comment: ""
        )
        label.textColor = .white
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

    private let activityIndicator: SpinnerActivityIndicatorView = {
        let activityIndicator = SpinnerActivityIndicatorView(style: .medium)
        activityIndicator.translatesAutoresizingMaskIntoConstraints = false
        activityIndicator.tintColor = .white
        return activityIndicator
    }()

    private let statusLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 17)
        label.textColor = UIColor.white.withAlphaComponent(0.6)
        label.translatesAutoresizingMaskIntoConstraints = false
        label.numberOfLines = 0
        label.alpha = 0
        return label
    }()

    private let redeemButton: AppButton = {
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

    private let cancelButton: AppButton = {
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
        let stackView = UIStackView(arrangedSubviews: [
            instructionLabel,
            inputTextField,
            statusStack,
        ])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.StackSpacing.close
        return stackView
    }()

    private lazy var bottomStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [redeemButton, cancelButton])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.StackSpacing.regular
        return stackView
    }()

    private let successImage: UIImageView = {
        let imageView = UIImageView(image: UIImage(named: "IconSuccess"))
        imageView.translatesAutoresizingMaskIntoConstraints = false
        imageView.contentMode = .scaleAspectFit
        imageView.alpha = 0
        return imageView
    }()

    private lazy var topStackTopConstraint: NSLayoutConstraint = topStackView.topAnchor.constraint(
        equalTo: successImage.bottomAnchor,
        constant: 0
    )

    private lazy var successImageHeightConstraint = NSLayoutConstraint(
        item: successImage,
        attribute: .height,
        relatedBy: .equal,
        toItem: nil,
        attribute: .notAnAttribute,
        multiplier: 1,
        constant: 0
    )

    // MARK: - Variables

    private let redeemAction: Action
    private let cancelAction: Action

    // MARK: - Lifecycles

    init(redeemAction: @escaping Action, cancelAction: @escaping Action) {
        self.redeemAction = redeemAction
        self.cancelAction = cancelAction

        super.init(frame: .zero)

        translatesAutoresizingMaskIntoConstraints = false
        backgroundColor = .secondaryColor
        layoutMargins = UIMetrics.contentLayoutMargins

        setUpSubviews()
        configureConstraints()
        subscribeClicks()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - View setup

    private func setUpSubviews() {
        addSubview(successImage)
        addSubview(topStackView)
        addSubview(bottomStackView)
    }

    private func configureConstraints() {
        NSLayoutConstraint.activate([
            successImage.topAnchor.constraint(equalTo: layoutMarginsGuide.topAnchor),
            successImage.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            successImage.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            successImageHeightConstraint,

            topStackTopConstraint,
            topStackView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            topStackView.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),

            bottomStackView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            bottomStackView.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            bottomStackView.bottomAnchor.constraint(equalTo: layoutMarginsGuide.bottomAnchor),
        ])
    }

    fileprivate func subscribeClicks() {
        cancelButton.addTarget(
            self,
            action: #selector(cancelButtonTapped),
            for: .touchUpInside
        )

        redeemButton.addTarget(
            self,
            action: #selector(RedeemButtonTapped),
            for: .touchUpInside
        )
    }

    // MARK: - Capabilities

    @objc private func cancelButtonTapped(_ sender: AppButton) {
        cancelAction()
    }

    @objc private func RedeemButtonTapped(_ sender: AppButton) {
        redeemAction()
    }

    /// Update views based on RedeemVoucherState that we get from online end point.
    /// - Parameters:
    ///   - state: RedeemVoucherViewController.RedeemVoucherState.
    ///   - isVoucherLengthSatisfied: true if Textfield length is equal to predict.
    func updateViews(
        state: RedeemVoucherViewController.RedeemVoucherState,
        isVoucherLengthSatisfied: Bool
    ) {
        if state.isWaiting {
            showLoading()
        } else {
            hideLoading()
        }

        setRedeemButtonAvailability(
            !state.isWaiting && isVoucherLengthSatisfied
        )

        updateStatusLabel(
            alpha: state == .initial ? 0 : 1,
            text: state.getStatusLabelText(),
            textColor: state == .failure ? .dangerColor : .white
        )

        if case .success = state {
            redeemedVoucherSuccessfully(
                instructionLabelSuccessString: instructionLabelSuccessString,
                gotItButtonTitle: gotItButtonTitle
            )
        }
    }
}

// MARK: - Private functions

//// Used for mainly updating views

private extension RedeemVoucherContentView {
    private func redeemedVoucherSuccessfully(
        instructionLabelSuccessString: String,
        gotItButtonTitle: String
    ) {
        inputTextField.isHidden = true
        topStackView.layoutIfNeeded()

        redeemButton.isHidden = true
        bottomStackView.layoutIfNeeded()

        instructionLabel.alpha = 1
        instructionLabel.text = instructionLabelSuccessString
        instructionLabel.font = UIFont.boldSystemFont(ofSize: 20)

        topStackView.spacing = UIMetrics.StackSpacing.close / 2
        topStackTopConstraint.constant = UIMetrics.sectionSpacing

        successImageHeightConstraint.constant
            = SpinnerActivityIndicatorView.Style.large.intrinsicSize.height
        successImage.alpha = 1

        statusLabel.alpha = 0.6

        cancelButton.setTitle(gotItButtonTitle, for: .normal)

        topStackView.spacing = UIMetrics.StackSpacing.close
        inputTextField.removeFromSuperview()
        redeemButton.removeFromSuperview()
    }

    private func showLoading() {
        activityIndicator.alpha = 1
        activityIndicator.startAnimating()
    }

    private func hideLoading() {
        activityIndicator.stopAnimating()
        activityIndicator.alpha = 0
    }

    private func setRedeemButtonAvailability(_ isAvailable: Bool) {
        redeemButton.isEnabled = isAvailable
    }

    private func updateStatusLabel(alpha: CGFloat, text: String, textColor: UIColor) {
        statusLabel.alpha = alpha
        statusLabel.text = text
        statusLabel.textColor = textColor
    }
}
