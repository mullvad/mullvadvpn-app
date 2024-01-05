//
//  AccountDeletionContentView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import UIKit

protocol AccountDeletionContentViewDelegate: AnyObject {
    func didTapDeleteButton(contentView: AccountDeletionContentView, button: AppButton)
    func didTapCancelButton(contentView: AccountDeletionContentView, button: AppButton)
}

class AccountDeletionContentView: UIView {
    enum State {
        case initial
        case loading
        case failure(Error)
    }

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
        label.translatesAutoresizingMaskIntoConstraints = false
        label.font = .preferredFont(forTextStyle: .title2, weight: .bold)
        label.numberOfLines = .zero
        label.lineBreakMode = .byWordWrapping
        label.textColor = .white
        label.text = NSLocalizedString(
            "ACCOUNT_DELETION_PAGE_TITLE",
            tableName: "Account",
            value: "Account deletion",
            comment: ""
        )
        return label
    }()

    private let messageLabel: UILabel = {
        let label = UILabel()
        label.translatesAutoresizingMaskIntoConstraints = false
        label.font = .preferredFont(forTextStyle: .body, weight: .bold)
        label.numberOfLines = .zero
        label.lineBreakMode = .byWordWrapping
        label.textColor = .white
        return label
    }()

    private let tipLabel: UILabel = {
        let label = UILabel()
        label.translatesAutoresizingMaskIntoConstraints = false
        label.font = .preferredFont(forTextStyle: .footnote, weight: .bold)
        label.numberOfLines = .zero
        label.lineBreakMode = .byWordWrapping
        label.textColor = .white
        label.text = NSLocalizedString(
            "TIP_TEXT",
            tableName: "Account",
            value: """
            This logs out all devices using this account and all \
            VPN access will be denied even if there is time left on the account. \
            Enter the last 4 digits of the account number and hit "Delete account" if you really want to delete the account :
            """,
            comment: ""
        )
        return label
    }()

    private lazy var accountTextField: AccountTextField = {
        let groupingStyle = AccountTextField.GroupingStyle.lastPart
        let textField = AccountTextField(groupingStyle: groupingStyle)
        textField.font = .preferredFont(forTextStyle: .body, weight: .bold)
        textField.placeholder = Array(repeating: "X", count: 4).joined()
        textField.placeholderTextColor = .lightGray
        textField.textContentType = .username
        textField.autocorrectionType = .no
        textField.smartDashesType = .no
        textField.smartInsertDeleteType = .no
        textField.smartQuotesType = .no
        textField.spellCheckingType = .no
        textField.keyboardType = .numberPad
        textField.returnKeyType = .done
        textField.enablesReturnKeyAutomatically = false
        textField.backgroundColor = .white
        textField.borderStyle = .line
        return textField
    }()

    private let deleteButton: AppButton = {
        let button = AppButton(style: .danger)
        button.accessibilityIdentifier = .deleteButton
        button.setTitle(NSLocalizedString(
            "DELETE_ACCOUNT_BUTTON_TITLE",
            tableName: "Account",
            value: "Delete Account",
            comment: ""
        ), for: .normal)
        return button
    }()

    private let cancelButton: AppButton = {
        let button = AppButton(style: .default)
        button.accessibilityIdentifier = .cancelButton
        button.setTitle(NSLocalizedString(
            "CANCEL_BUTTON_TITLE",
            tableName: "Account",
            value: "Cancel",
            comment: ""
        ), for: .normal)
        return button
    }()

    private lazy var textsStack: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [
            titleLabel,
            messageLabel,
            tipLabel,
            accountTextField,
            statusStack,
        ])
        stackView.setCustomSpacing(UIMetrics.padding8, after: titleLabel)
        stackView.setCustomSpacing(UIMetrics.padding16, after: messageLabel)
        stackView.setCustomSpacing(UIMetrics.padding8, after: tipLabel)
        stackView.setCustomSpacing(UIMetrics.padding4, after: accountTextField)
        stackView.setContentHuggingPriority(.defaultLow, for: .vertical)
        stackView.axis = .vertical
        return stackView
    }()

    private lazy var buttonsStack: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [deleteButton, cancelButton])
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.padding16
        stackView.setContentCompressionResistancePriority(.required, for: .vertical)
        return stackView
    }()

    private let activityIndicator: SpinnerActivityIndicatorView = {
        let activityIndicator = SpinnerActivityIndicatorView(style: .medium)
        activityIndicator.translatesAutoresizingMaskIntoConstraints = false
        activityIndicator.tintColor = .white
        activityIndicator.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        activityIndicator.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        return activityIndicator
    }()

    private let statusLabel: UILabel = {
        let label = UILabel()
        label.font = .preferredFont(forTextStyle: .body)
        label.numberOfLines = 2
        label.lineBreakMode = .byWordWrapping
        label.textColor = .red
        label.setContentHuggingPriority(.defaultLow, for: .horizontal)
        label.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        label.lineBreakStrategy = []
        return label
    }()

    private lazy var statusStack: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [activityIndicator, statusLabel])
        stackView.axis = .horizontal
        stackView.spacing = UIMetrics.padding8
        return stackView
    }()

    private var keyboardResponder: AutomaticKeyboardResponder?
    private var bottomsOfButtonsConstraint: NSLayoutConstraint?

    var state: State = .initial {
        didSet {
            updateUI()
        }
    }

    var isEditing: Bool {
        get {
            accountTextField.isEditing
        }
        set {
            guard accountTextField.isFirstResponder != newValue else { return }
            if newValue {
                accountTextField.becomeFirstResponder()
            } else {
                accountTextField.resignFirstResponder()
            }
        }
    }

    var viewModel: AccountDeletionViewModel? {
        didSet {
            updateData()
        }
    }

    var lastPartOfAccountNumber: String {
        accountTextField.parsedToken
    }

    private var text: String {
        switch state {
        case let .failure(error):
            return error.localizedDescription
        case .loading:
            return NSLocalizedString(
                "DELETE_ACCOUNT_STATUS_WAITING",
                tableName: "Account",
                value: "Deleting account...",
                comment: ""
            )
        default: return ""
        }
    }

    private var isDeleteButtonEnabled: Bool {
        switch state {
        case .initial, .failure:
            return true
        case .loading:
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
        case .loading:
            return true
        default:
            return false
        }
    }

    private var isInputValid: Bool {
        guard let input = accountTextField.text,
              let accountNumber = viewModel?.accountNumber,
              !accountNumber.isEmpty
        else {
            return false
        }

        let inputLengthIsValid = input.count == 4
        let inputMatchesAccountNumber = accountNumber.suffix(4) == input

        return inputLengthIsValid && inputMatchesAccountNumber
    }

    weak var delegate: AccountDeletionContentViewDelegate?

    override init(frame: CGRect) {
        super.init(frame: .zero)
        commonInit()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        commonInit()
    }

    private func commonInit() {
        setupAppearance()
        configureUI()
        addActions()
        updateUI()
        addKeyboardResponder()
        addObservers()
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
        contentHolderView.addConstrainedSubviews([textsStack, buttonsStack]) {
            textsStack.pinEdgesToSuperview(.all().excluding(.bottom))
            buttonsStack.pinEdgesToSuperview(PinnableEdges([.leading(.zero), .trailing(.zero)]))
            textsStack.bottomAnchor.constraint(
                lessThanOrEqualTo: buttonsStack.topAnchor,
                constant: -UIMetrics.padding16
            )
        }
        bottomsOfButtonsConstraint = buttonsStack.pinEdgesToSuperview(PinnableEdges([.bottom(.zero)])).first
        bottomsOfButtonsConstraint?.isActive = true
    }

    private func addActions() {
        [deleteButton, cancelButton].forEach { $0.addTarget(
            self,
            action: #selector(didPress(button:)),
            for: .touchUpInside
        ) }
    }

    private func updateData() {
        viewModel.flatMap { viewModel in
            let text = NSLocalizedString(
                "BODY_LABEL_TEXT",
                tableName: "Account",
                value: """
                Are you sure you want to delete account **\(viewModel.accountNumber)**?
                """,
                comment: ""
            )
            messageLabel.attributedText = NSAttributedString(
                markdownString: text,
                options: MarkdownStylingOptions(
                    font: .preferredFont(forTextStyle: .body)
                )
            )
        }
    }

    private func updateUI() {
        if isLoading {
            activityIndicator.startAnimating()
        } else {
            activityIndicator.stopAnimating()
        }
        deleteButton.isEnabled = isDeleteButtonEnabled && isInputValid
        statusLabel.text = text
        statusLabel.textColor = textColor
    }

    private func setupAppearance() {
        translatesAutoresizingMaskIntoConstraints = false
        backgroundColor = .secondaryColor
        directionalLayoutMargins = UIMetrics.contentLayoutMargins
    }

    private func addKeyboardResponder() {
        keyboardResponder = AutomaticKeyboardResponder(
            targetView: self,
            handler: { [weak self] _, offset in
                guard let self else { return }
                self.bottomsOfButtonsConstraint?.constant = isEditing ? -offset : 0
                self.layoutIfNeeded()
                self.scrollView.flashScrollIndicators()
            }
        )
    }

    private func addObservers() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(textDidChange),
            name: UITextField.textDidChangeNotification,
            object: accountTextField
        )
    }

    @objc private func didPress(button: AppButton) {
        switch AccessibilityIdentifier(rawValue: button.accessibilityIdentifier ?? "") {
        case .deleteButton:
            delegate?.didTapDeleteButton(contentView: self, button: button)
        case .cancelButton:
            delegate?.didTapCancelButton(contentView: self, button: button)
        default: return
        }
    }

    @objc private func textDidChange() {
        updateUI()
    }
}
