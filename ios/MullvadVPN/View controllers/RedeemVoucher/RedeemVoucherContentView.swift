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
    case logout
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

    private let voucherTextFieldHeight: CGFloat = 54

    private let title: UILabel = {
        let label = UILabel()
        label.font = .preferredFont(forTextStyle: .title1, weight: .bold).withSize(32)
        label.text = NSLocalizedString(
            "REDEEM_VOUCHER_TITLE",
            tableName: "RedeemVoucher",
            value: "Redeem voucher",
            comment: ""
        )
        label.textColor = .white
        label.numberOfLines = 0
        return label
    }()

    private let enterVoucherLabel: UILabel = {
        let label = UILabel()
        label.font = .preferredFont(forTextStyle: .body, weight: .semibold).withSize(15)

        label.text = NSLocalizedString(
            "REDEEM_VOUCHER_INSTRUCTION",
            tableName: "RedeemVoucher",
            value: "Enter voucher code",
            comment: ""
        )
        label.textColor = .white
        label.numberOfLines = 0
        return label
    }()

    private let textField: VoucherTextField = {
        let textField = VoucherTextField()
        textField.font = UIFont.monospacedSystemFont(ofSize: 15, weight: .regular)
        textField.placeholder = Array(repeating: "XXXX", count: 4).joined(separator: "-")
        textField.placeholderTextColor = .lightGray
        textField.backgroundColor = .white
        textField.cornerRadius = UIMetrics.SettingsRedeemVoucher.cornerRadius
        textField.keyboardType = .asciiCapable
        textField.autocapitalizationType = .allCharacters
        textField.returnKeyType = .done
        textField.autocorrectionType = .no
        return textField
    }()

    private let activityIndicator: SpinnerActivityIndicatorView = {
        let activityIndicator = SpinnerActivityIndicatorView(style: .medium)
        activityIndicator.tintColor = .white
        activityIndicator.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        activityIndicator.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        return activityIndicator
    }()

    private let statusLabel: UILabel = {
        let label = UILabel()
        label.font = .systemFont(ofSize: 13, weight: .semibold)
        label.numberOfLines = 2
        label.lineBreakMode = .byWordWrapping
        label.textColor = .red
        label.setContentHuggingPriority(.defaultLow, for: .horizontal)
        label.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        label.lineBreakStrategy = []
        return label
    }()

    private lazy var logoutViewForAccountNumberIsEntered: LogoutDialogueView = {
        LogoutDialogueView { verifiedAccountView in
            verifiedAccountView.isLoading = true
            self.logoutAction?()
        }
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
        stackView.spacing = UIMetrics.padding8
        return stackView
    }()

    private lazy var voucherCodeStackView: UIStackView = {
        var arrangedSubviews = [
            enterVoucherLabel,
            textField,
            statusStack,
            logoutViewForAccountNumberIsEntered,
        ]

        if configuration.shouldUseCompactStyle == false {
            arrangedSubviews.insert(title, at: 0)
        }

        let stackView = UIStackView(arrangedSubviews: arrangedSubviews)
        stackView.axis = .vertical
        stackView.setCustomSpacing(UIMetrics.padding8, after: title)
        stackView.setCustomSpacing(UIMetrics.padding16, after: enterVoucherLabel)
        stackView.setCustomSpacing(UIMetrics.padding8, after: textField)
        stackView.setCustomSpacing(UIMetrics.padding16, after: statusLabel)
        stackView.setCustomSpacing(UIMetrics.padding10, after: statusStack)
        stackView.setContentHuggingPriority(.defaultLow, for: .vertical)

        return stackView
    }()

    private lazy var buttonsStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [redeemButton, cancelButton])
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
        case .logout:
            return NSLocalizedString(
                "REDEEM_VOUCHER_STATUS_WAITING",
                tableName: "RedeemVoucher",
                value: "Logging out...",
                comment: ""
            )
        default: return ""
        }
    }

    private var isRedeemButtonEnabled: Bool {
        switch state {
        case .initial, .failure:
            return true
        case .success, .verifying, .logout:
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
        case .verifying, .logout:
            return true
        default:
            return false
        }
    }

    private var keyboardResponder: AutomaticKeyboardResponder?
    private var bottomsOfButtonsConstraint: NSLayoutConstraint?
    private let configuration: RedeemVoucherViewConfiguration

    // MARK: - public

    var redeemAction: ((String) -> Void)?
    var cancelAction: (() -> Void)?
    var logoutAction: (() -> Void)?

    var state: RedeemVoucherState = .initial {
        didSet {
            updateUI()
        }
    }

    var isEditing: Bool {
        get {
            textField.isEditing
        }
        set {
            guard textField.isFirstResponder != newValue else { return }
            if newValue {
                textField.becomeFirstResponder()
            } else {
                textField.resignFirstResponder()
            }
        }
    }

    var isLogoutDialogHidden = true {
        didSet {
            logoutViewForAccountNumberIsEntered.isHidden = isLogoutDialogHidden
        }
    }

    init(configuration: RedeemVoucherViewConfiguration) {
        self.configuration = configuration
        super.init(frame: .zero)
        commonInit()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func commonInit() {
        setupAppearance()
        configureUI()
        addButtonHandlers()
        updateUI()
        addKeyboardResponderIfNeeded()
        addObservers()
    }

    private func setupAppearance() {
        translatesAutoresizingMaskIntoConstraints = false
        backgroundColor = .secondaryColor
        directionalLayoutMargins = UIMetrics.contentLayoutMargins
    }

    private func configureUI() {
        addConstrainedSubviews([scrollView]) {
            scrollView.pinEdgesToSuperview(.all(configuration.layoutMargins))
        }

        scrollView.addConstrainedSubviews([contentHolderView]) {
            contentHolderView.pinEdgesToSuperview()
            contentHolderView.widthAnchor.constraint(equalTo: scrollView.widthAnchor)
            contentHolderView.heightAnchor.constraint(greaterThanOrEqualTo: scrollView.heightAnchor)
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
        if isLoading {
            activityIndicator.startAnimating()
        } else {
            activityIndicator.stopAnimating()
        }
        redeemButton.isEnabled = isRedeemButtonEnabled && textField.isVoucherLengthSatisfied
        statusLabel.text = text
        statusLabel.textColor = textColor
        logoutViewForAccountNumberIsEntered.isLoading = isLoading
    }

    private func addObservers() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(textDidChange),
            name: UITextField.textDidChangeNotification,
            object: textField
        )
    }

    @objc private func cancelButtonTapped(_ sender: AppButton) {
        cancelAction?()
    }

    @objc private func redeemButtonTapped(_ sender: AppButton) {
        let code = textField.parsedToken
        guard !code.isEmpty else {
            return
        }
        redeemAction?(code)
    }

    @objc private func textDidChange() {
        if textField.parsedToken.isEmpty {
            isLogoutDialogHidden = true
        }
        updateUI()
    }

    private func addKeyboardResponderIfNeeded() {
        guard configuration.adjustViewWhenKeyboardAppears else { return }
        keyboardResponder = AutomaticKeyboardResponder(
            targetView: self,
            handler: { [weak self] _, offset in
                guard let self else { return }
                self.bottomsOfButtonsConstraint?.constant = isEditing ? -offset : 0
                self.layoutIfNeeded()
            }
        )
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
