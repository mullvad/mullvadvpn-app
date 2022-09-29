//
//  RedeemVoucherInputContentView.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-05.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum RedeemVoucherState {
    case initial
    case success
    case verifying
    case failure(Error)

    fileprivate var statusText: String? {
        switch self {
        case .initial, .success:
            return nil

        case let .failure(error):
            guard let restError = error as? REST.Error else {
                return error.localizedDescription
            }

            if restError.compareErrorCode(.invalidVoucher) {
                return NSLocalizedString(
                    "REDEEM_VOUCHER_STATUS_FAILURE",
                    tableName: "RedeemVoucher",
                    value: "Voucher code is invalid.",
                    comment: ""
                )
            } else {
                return restError.errorChainDescription
            }

        case .verifying:
            return NSLocalizedString(
                "REDEEM_VOUCHER_STATUS_WAITING",
                tableName: "RedeemVoucher",
                value: "Verifying voucher...",
                comment: ""
            )
        }
    }

    fileprivate var shouldEnableRedeemButton: Bool {
        switch self {
        case .initial, .failure:
            return true
        case .success, .verifying:
            return false
        }
    }

    fileprivate var statusTextColor: UIColor {
        switch self {
        case .failure:
            return .dangerColor
        default:
            return .white
        }
    }
}

class RedeemVoucherInputContentView: UIView {
    var state: RedeemVoucherState = .initial {
        didSet {
            updateSubviews()
        }
    }

    private let titleLabel: UILabel = {
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

    let textField: VoucherTextField = {
        let textField = VoucherTextField()
        textField.font = UIFont.monospacedSystemFont(ofSize: 20, weight: .regular)
        textField.translatesAutoresizingMaskIntoConstraints = false
        textField.placeholder = "XXXX-XXXX-XXXX-XXXX"
        textField.placeholderTextColor = .lightGray
        textField.backgroundColor = .white
        textField.cornerRadius = 8
        textField.keyboardType = .asciiCapable
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
        label.translatesAutoresizingMaskIntoConstraints = false
        label.font = UIFont.systemFont(ofSize: 17)
        label.textColor = .white
        label.numberOfLines = 0
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
        stackView.spacing = UIMetrics.StackSpacing.close
        return stackView
    }()

    private lazy var topStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [
            titleLabel,
            textField,
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

    var redeemAction: (() -> Void)?
    var cancelAction: (() -> Void)?

    init() {
        super.init(frame: .zero)

        translatesAutoresizingMaskIntoConstraints = false
        backgroundColor = .secondaryColor
        layoutMargins = UIMetrics.contentLayoutMargins

        addSubview(topStackView)
        addSubview(bottomStackView)

        addConstraints()
        addButtonHandlers()
        addTextFieldObserver()

        updateSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func addConstraints() {
        NSLayoutConstraint.activate([
            topStackView.topAnchor.constraint(equalTo: layoutMarginsGuide.topAnchor),
            topStackView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            topStackView.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),

            bottomStackView.topAnchor.constraint(
                greaterThanOrEqualTo: topStackView.bottomAnchor,
                constant: UIMetrics.StackSpacing.regular
            ),
            bottomStackView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            bottomStackView.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            bottomStackView.bottomAnchor.constraint(equalTo: layoutMarginsGuide.bottomAnchor),
        ])
    }

    private func addTextFieldObserver() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(textDidChange),
            name: UITextField.textDidChangeNotification,
            object: textField
        )
    }

    private func addButtonHandlers() {
        cancelButton.addTarget(
            self,
            action: #selector(cancelButtonTapped),
            for: .touchUpInside
        )

        redeemButton.addTarget(
            self,
            action: #selector(redeemButtonTapped),
            for: .touchUpInside
        )
    }

    private func updateSubviews() {
        if case .verifying = state {
            activityIndicator.startAnimating()
        } else {
            activityIndicator.stopAnimating()
        }

        redeemButton.isEnabled = state.shouldEnableRedeemButton && textField
            .satisfiesVoucherLengthRequirement
        statusLabel.text = state.statusText
        statusLabel.textColor = state.statusTextColor
    }

    @objc private func cancelButtonTapped(_ sender: AppButton) {
        cancelAction?()
    }

    @objc private func redeemButtonTapped(_ sender: AppButton) {
        redeemAction?()
    }

    @objc private func textDidChange() {
        updateSubviews()
    }
}
