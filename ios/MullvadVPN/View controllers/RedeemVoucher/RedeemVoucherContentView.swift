//
//  RedeemVoucherContentView.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-05.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import UIKit

enum RedeemVoucherState {
    case initial
    case success
    case verifying
    case failure(Error)
}

final class RedeemVoucherContentView: UIView {
    // MARK: - private

    private let scrollView: UIScrollView = {
        let scrollView = UIScrollView()
        return scrollView
    }()

    private let contentHolderView: UIView = {
        let contentHolderView = UIView()
        return contentHolderView
    }()

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

    private let textField: VoucherTextField = {
        let textField = VoucherTextField()
        textField.font = UIFont.monospacedSystemFont(ofSize: 15, weight: .regular)
        textField.placeholder = Array(repeating: "XXXX", count: 4).joined(separator: "-")
        textField.placeholderTextColor = .lightGray
        textField.backgroundColor = .white
        textField.cornerRadius = UIMetrics.RedeemVoucher.cornerRadius
        textField.keyboardType = .asciiCapable
        textField.autocapitalizationType = .allCharacters
        textField.returnKeyType = .done
        textField.autocorrectionType = .no
        return textField
    }()

    private let activityIndicator: SpinnerActivityIndicatorView = {
        let activityIndicator = SpinnerActivityIndicatorView(style: .medium)
        activityIndicator.tintColor = .white
        return activityIndicator
    }()

    private let statusLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 17)
        label.textColor = .white
        label.numberOfLines = .zero
        label.lineBreakMode = .byWordWrapping
        if #available(iOS 14.0, *) {
            // See: https://stackoverflow.com/q/46200027/351305
            label.lineBreakStrategy = []
        }
        return label
    }()

    private let redeemButton: AppButton = {
        let button = AppButton(style: .success)
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
        stackView.spacing = UIMetrics.interButtonSpacing
        return stackView
    }()

    private lazy var voucherCodeStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [
            titleLabel,
            textField,
            statusStack,
        ])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.setCustomSpacing(UIMetrics.padding16, after: titleLabel)
        stackView.setCustomSpacing(UIMetrics.padding8, after: textField)
        stackView.setCustomSpacing(UIMetrics.padding16, after: statusLabel)
        stackView.setContentHuggingPriority(.defaultLow, for: .vertical)
        return stackView
    }()

    private lazy var buttonsStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [redeemButton, cancelButton])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.padding16
        stackView.setContentCompressionResistancePriority(.required, for: .vertical)
        return stackView
    }()

    private var text: String {
        switch state {
        case let .failure(error):
            guard let restError = error as? REST.Error else {
                return error.localizedDescription
            }
            return restError.description
        case .verifying:
            return NSLocalizedString(
                "REDEEM_VOUCHER_STATUS_WAITING",
                tableName: "RedeemVoucher",
                value: "Verifying voucher...",
                comment: ""
            )
        default: return ""
        }
    }

    private var isRedeemButtonEnabled: Bool {
        switch state {
        case .initial, .failure:
            return true
        case .success, .verifying:
            return false
        }
    }

    private var textColor: UIColor {
        switch state {
        case .failure:
            return .dangerColor
        default:
            return .white
        }
    }

    private var isLoading: Bool {
        switch state {
        case .verifying:
            return true
        default:
            return false
        }
    }

    private var keyboardResponder: AutomaticKeyboardResponder?
    private var bottomsOfButtonsConstraint: NSLayoutConstraint?

    // MARK: - public

    var redeemAction: ((String) -> Void)?
    var cancelAction: (() -> Void)?

    var state: RedeemVoucherState = .initial {
        didSet {
            updateUI()
        }
    }

    var isEditing = false {
        didSet {
            if isEditing {
                textField.becomeFirstResponder()
            } else {
                textField.resignFirstResponder()
            }
        }
    }

    init() {
        super.init(frame: .zero)
        setup()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setup() {
        setupAppearance()
        configureUI()
        addButtonHandlers()
        addTextFieldObserver()
        updateUI()
        addKeyboardResponder()
    }

    private func configureUI() {
        addConstrainedSubviews([scrollView]) {
            scrollView.pinEdgesToSuperviewMargins()
        }

        scrollView.addConstrainedSubviews([contentHolderView]) {
            contentHolderView.pinEdgesToSuperview()
            contentHolderView.widthAnchor.constraint(equalTo: scrollView.widthAnchor, multiplier: 1.0)
            contentHolderView.heightAnchor.constraint(greaterThanOrEqualTo: scrollView.heightAnchor, multiplier: 1.0)
        }
        contentHolderView.addConstrainedSubviews([voucherCodeStackView, buttonsStackView]) {
            voucherCodeStackView.pinEdgesToSuperview(.all().excluding(.bottom))
            buttonsStackView.pinEdgesToSuperview(PinnableEdges([.leading(.zero), .trailing(.zero)]))
            voucherCodeStackView.bottomAnchor.constraint(
                lessThanOrEqualTo: buttonsStackView.topAnchor,
                constant: -UIMetrics.padding16
            )
        }
        bottomsOfButtonsConstraint = buttonsStackView.pinEdgesToSuperview(PinnableEdges([.bottom(.zero)])).first
        bottomsOfButtonsConstraint?.isActive = true
    }

    private func setupAppearance() {
        translatesAutoresizingMaskIntoConstraints = false
        backgroundColor = .secondaryColor
        directionalLayoutMargins = UIMetrics.contentLayoutMargins
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

    private func updateUI() {
        isLoading ? activityIndicator.startAnimating() : activityIndicator.stopAnimating()
        redeemButton.isEnabled = isRedeemButtonEnabled && textField.isVoucherLengthSatisfied
        statusLabel.text = text
        statusLabel.textColor = textColor
    }

    private func addKeyboardResponder() {
        keyboardResponder = AutomaticKeyboardResponder(
            targetView: self,
            handler: { targetView, offset in
                guard self.textField.isFirstResponder else { return }
                self.bottomsOfButtonsConstraint?.constant = -offset
                self.layoutIfNeeded()
            }
        )
    }

    @objc private func cancelButtonTapped(_ sender: AppButton) {
        cancelAction?()
    }

    @objc private func redeemButtonTapped(_ sender: AppButton) {
        guard let code = textField.text, !code.isEmpty else {
            return
        }
        redeemAction?(code)
    }

    @objc private func textDidChange() {
        updateUI()
    }
}

private extension REST.Error {
    var description: String {
        if compareErrorCode(.invalidVoucher) {
            return NSLocalizedString(
                "REDEEM_VOUCHER_STATUS_FAILURE",
                tableName: "RedeemVoucher",
                value: "Voucher code is invalid.",
                comment: ""
            )
        } else if compareErrorCode(.usedVoucher) {
            return NSLocalizedString(
                "REDEEM_VOUCHER_STATUS_FAILURE",
                tableName: "RedeemVoucher",
                value: "This voucher code has already been used.",
                comment: ""
            )
        }
        return displayErrorDescription ?? ""
    }
}
