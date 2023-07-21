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
    func didTapDeleteButtonButton(contentView: AccountDeletionContentView, button: AppButton)
    func didTapCancelButton(contentView: AccountDeletionContentView, button: AppButton)
}

class AccountDeletionContentView: UIView {
    enum State {
        case initial
        case loading
        case failure(Error)
    }

    private enum Action: String {
        case ok, cancel
    }

    private let titleLabel: UILabel = {
        let label = UILabel()
        label.translatesAutoresizingMaskIntoConstraints = false
        label.font = .preferredFont(forTextStyle: .title3, weight: .bold)
        label.numberOfLines = .zero
        label.lineBreakMode = .byWordWrapping
        label.textColor = .white
        label.text = NSLocalizedString(
            "TITLE",
            tableName: "Account",
            value: "Account deletion",
            comment: ""
        )
        return label
    }()

    private let messageLabel: UILabel = {
        let label = UILabel()
        label.translatesAutoresizingMaskIntoConstraints = false
        label.font = .preferredFont(forTextStyle: .body, weight: .regular)
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
            This logs out all devices using this account and all VPN access will be denied even if there is time left on the account. Enter the last 4 digits of the account number and hit OK if you really want to delete the account :
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

    private let statusLabel: UILabel = {
        let label = UILabel()
        label.font = .preferredFont(forTextStyle: .footnote)
        label.textColor = .white
        label.numberOfLines = 2
        label.lineBreakMode = .byWordWrapping
        label.textColor = .red
        if #available(iOS 14.0, *) {
            // See: https://stackoverflow.com/q/46200027/351305
            label.lineBreakStrategy = []
        }
        return label
    }()

    private let deleteButton: AppButton = {
        let button = AppButton(style: .danger)
        button.accessibilityIdentifier = Action.ok.rawValue
        button.setTitle(NSLocalizedString(
            "OK_BUTTON_TITLE",
            tableName: "Account",
            value: "Ok",
            comment: ""
        ), for: .normal)
        return button
    }()

    private let cancelButton: AppButton = {
        let button = AppButton(style: .default)
        button.accessibilityIdentifier = Action.cancel.rawValue
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
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.setCustomSpacing(UIMetrics.padding4, after: titleLabel)
        stackView.setCustomSpacing(UIMetrics.padding8, after: messageLabel)
        stackView.setCustomSpacing(UIMetrics.padding8, after: tipLabel)
        stackView.setCustomSpacing(UIMetrics.padding4, after: accountTextField)
        stackView.axis = .vertical
        return stackView
    }()

    private lazy var buttonsStack: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [deleteButton, cancelButton])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.padding8
        return stackView
    }()

    private let activityIndicator: SpinnerActivityIndicatorView = {
        let activityIndicator = SpinnerActivityIndicatorView(style: .medium)
        activityIndicator.tintColor = .white
        return activityIndicator
    }()

    private lazy var statusStack: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [activityIndicator, statusLabel])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .horizontal
        stackView.spacing = UIMetrics.interButtonSpacing
        return stackView
    }()

    var state: State = .initial {
        didSet {
            updateUI()
        }
    }

    var isEditing = false {
        didSet {
            _ = accountTextField.isFirstResponder
                ? accountTextField.resignFirstResponder()
                : accountTextField.becomeFirstResponder()
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

    private var isAccountNumberLengthSatisfied: Bool {
        let length = accountTextField.text?.count ?? 0
        return length == 4
    }

    weak var delegate: AccountDeletionContentViewDelegate?

    override init(frame: CGRect) {
        super.init(frame: .zero)
        setupAppearance()
        configureUI()
        addActions()
        updateUI()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func willMove(toWindow newWindow: UIWindow?) {
        if newWindow == nil {
            NotificationCenter.default.removeObserver(self)
        }
    }

    override func didMoveToWindow() {
        if self.window != nil {
            NotificationCenter.default.addObserver(
                self,
                selector: #selector(textDidChange),
                name: UITextField.textDidChangeNotification,
                object: accountTextField
            )
        }
    }

    private func configureUI() {
        addConstrainedSubviews([textsStack, buttonsStack]) {
            textsStack.pinEdgesToSuperviewMargins(.all().excluding(.bottom))
            buttonsStack.pinEdgesToSuperviewMargins(.all().excluding(.top))
        }
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
                options: NSAttributedString
                    .MarkdownStylingOptions(
                        font: .preferredFont(forTextStyle: .body)
                    )
            )
        }
    }

    private func updateUI() {
        isLoading ? activityIndicator.startAnimating() : activityIndicator.stopAnimating()
        deleteButton.isEnabled = isDeleteButtonEnabled && isAccountNumberLengthSatisfied
        statusLabel.text = text
        statusLabel.textColor = textColor
    }

    private func setupAppearance() {
        translatesAutoresizingMaskIntoConstraints = false
        backgroundColor = .secondaryColor
        directionalLayoutMargins = UIMetrics.contentLayoutMargins
    }

    @objc private func didPress(button: AppButton) {
        switch Action(rawValue: button.accessibilityIdentifier ?? "") {
        case .ok:
            delegate?.didTapDeleteButtonButton(contentView: self, button: button)
        case .cancel:
            delegate?.didTapCancelButton(contentView: self, button: button)
        default: return
        }
    }

    @objc private func textDidChange() {
        updateUI()
    }
}
