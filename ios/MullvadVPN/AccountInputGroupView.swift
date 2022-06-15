//
//  AccountInputGroupView.swift
//  MullvadVPN
//
//  Created by pronebird on 22/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit
import Logging

protocol AccountInputGroupViewDelegate: AnyObject {
    func accountInputGroupViewShouldRemoveLastUsedAccount(_ view: AccountInputGroupView) -> Bool
    func accountInputGroupViewShouldAttemptLogin(_ view: AccountInputGroupView)
}

class AccountInputGroupView: UIView {

    enum Style {
        case normal, error, authenticating
    }

    weak var delegate: AccountInputGroupViewDelegate?

    let sendButton: UIButton = {
        let button = UIButton(type: .custom)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setImage(UIImage(named: "IconArrow"), for: .normal)
        button.accessibilityLabel = NSLocalizedString(
            "ACCOUNT_INPUT_LOGIN_BUTTON_ACCESSIBILITY_LABEL",
            tableName: "AccountInput",
            value: "Log in",
            comment: ""
        )
        return button
    }()

    var textField: UITextField {
        return privateTextField
    }

    var parsedToken: String {
        return privateTextField.parsedToken
    }

    let minimumAccountTokenLength = 10

    var satisfiesMinimumTokenLengthRequirement: Bool {
        return privateTextField.parsedToken.count > minimumAccountTokenLength
    }

    private let privateTextField: AccountTextField = {
        let textField = AccountTextField()
        textField.font = accountNumberFont()
        textField.translatesAutoresizingMaskIntoConstraints = false
        textField.placeholder = "0000 0000 0000 0000"
        textField.placeholderTextColor = .lightGray
        textField.textContentType = .username
        textField.clearButtonMode = .never
        textField.autocapitalizationType = .none
        textField.autocorrectionType = .no
        textField.smartDashesType = .no
        textField.smartInsertDeleteType = .no
        textField.smartQuotesType = .no
        textField.spellCheckingType = .no
        textField.keyboardType = .numberPad
        textField.returnKeyType = .done
        textField.enablesReturnKeyAutomatically = false

        return textField
    }()

    private let separator: UIView = {
        let separator = UIView()
        separator.translatesAutoresizingMaskIntoConstraints = false
        separator.backgroundColor = UIColor.AccountTextField.NormalState.borderColor
        return separator
    }()

    private let topRowView: UIView = {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        view.backgroundColor = .white

        return view
    }()

    private let bottomRowView: UIView = {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        view.backgroundColor = .white.withAlphaComponent(0.8)

        return view
    }()

    private let lastUsedAccountButton: UIButton = {
        let button = UIButton(type: .system)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.titleLabel?.font = accountNumberFont()
        button.setTitle("", for: .normal)
        button.contentHorizontalAlignment = .leading
        button.titleEdgeInsets = UIEdgeInsets(top: 0, left: UIMetrics.textFieldMargins.left, bottom: 0, right: 0)
        button.setTitleColor(UIColor.AccountTextField.NormalState.textColor, for: .normal)

        return button
    }()

    private let removeLastUsedAccountButton: UIButton = {
        let button = UIButton(type: .custom)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setImage(UIImage(named: "IconCloseSml"), for: .normal)
        button.imageView?.tintColor = .primaryColor.withAlphaComponent(0.4)

        return button
    }()

    private let contentView: UIView = {
        let view = UIView()
        view.backgroundColor = .clear
        view.translatesAutoresizingMaskIntoConstraints = false

        return view
    }()

    private(set) var loginState = LoginState.default

    private let borderRadius = CGFloat(8)
    private let borderWidth = CGFloat(2)

    private var lastUsedAccount: String = ""

    private var borderColor: UIColor {
        switch loginState {
        case .default:
            return privateTextField.isEditing
                             ? UIColor.AccountTextField.NormalState.borderColor
                             : UIColor.clear

        case .failure:
            return UIColor.AccountTextField.ErrorState.borderColor

        case .authenticating, .success:
            return UIColor.AccountTextField.AuthenticatingState.borderColor
        }
    }

    private var textColor: UIColor {
        switch loginState {
        case .default:
            return UIColor.AccountTextField.NormalState.textColor

        case .failure:
            return UIColor.AccountTextField.ErrorState.textColor

        case .authenticating, .success:
            return UIColor.AccountTextField.AuthenticatingState.textColor
        }
    }

    private var backgroundLayerColor: UIColor {
        switch loginState {
        case .default:
            return UIColor.AccountTextField.NormalState.backgroundColor

        case .failure:
            return UIColor.AccountTextField.ErrorState.backgroundColor

        case .authenticating, .success:
            return UIColor.AccountTextField.AuthenticatingState.backgroundColor
        }
    }

    private let borderLayer = CAShapeLayer()
    private let contentLayerMask = CALayer()

    var lastUsedAccountViewHeightConstraint: NSLayoutConstraint!
    var lastUsedAccountHeightConstraint: NSLayoutConstraint!
    var separatorHeightConstraint: NSLayoutConstraint!

    // MARK: - View lifecycle

    override init(frame: CGRect) {
        super.init(frame: frame)

        addSubview(contentView)
        contentView.addSubview(topRowView)
        contentView.addSubview(bottomRowView)
        topRowView.addSubview(privateTextField)
        topRowView.addSubview(sendButton)
        bottomRowView.addSubview(separator)
        bottomRowView.addSubview(lastUsedAccountButton)
        bottomRowView.addSubview(removeLastUsedAccountButton)

        privateTextField.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        sendButton.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)

        separatorHeightConstraint = separator.heightAnchor.constraint(equalToConstant: 0)
        lastUsedAccountHeightConstraint = lastUsedAccountButton.heightAnchor.constraint(equalToConstant: 0)

        lastUsedAccountButton.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        removeLastUsedAccountButton.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: topAnchor),
            contentView.leadingAnchor.constraint(equalTo: leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: bottomAnchor),

            topRowView.topAnchor.constraint(equalTo: contentView.topAnchor),
            topRowView.leadingAnchor.constraint(equalTo: contentView.leadingAnchor),
            topRowView.trailingAnchor.constraint(equalTo: contentView.trailingAnchor),
            topRowView.bottomAnchor.constraint(equalTo: bottomRowView.topAnchor),

            bottomRowView.topAnchor.constraint(equalTo: topRowView.bottomAnchor),
            bottomRowView.leadingAnchor.constraint(equalTo: contentView.leadingAnchor),
            bottomRowView.trailingAnchor.constraint(equalTo: contentView.trailingAnchor),
            bottomRowView.bottomAnchor.constraint(equalTo: contentView.bottomAnchor),

            privateTextField.topAnchor.constraint(equalTo: topRowView.topAnchor),
            privateTextField.leadingAnchor.constraint(equalTo: topRowView.leadingAnchor),
            privateTextField.trailingAnchor.constraint(equalTo: sendButton.leadingAnchor),
            privateTextField.bottomAnchor.constraint(equalTo: topRowView.bottomAnchor),

            sendButton.topAnchor.constraint(equalTo: topRowView.topAnchor),
            sendButton.trailingAnchor.constraint(equalTo: topRowView.trailingAnchor),
            sendButton.bottomAnchor.constraint(equalTo: topRowView.bottomAnchor),
            sendButton.widthAnchor.constraint(equalTo: sendButton.heightAnchor),

            separator.topAnchor.constraint(equalTo: bottomRowView.topAnchor),
            separator.bottomAnchor.constraint(equalTo: lastUsedAccountButton.topAnchor),
            separator.leadingAnchor.constraint(equalTo: bottomRowView.leadingAnchor),
            separator.trailingAnchor.constraint(equalTo: bottomRowView.trailingAnchor),
            separatorHeightConstraint,

            lastUsedAccountButton.topAnchor.constraint(equalTo: separator.bottomAnchor),
            lastUsedAccountButton.bottomAnchor.constraint(equalTo: bottomRowView.bottomAnchor),
            lastUsedAccountButton.leadingAnchor.constraint(equalTo: bottomRowView.leadingAnchor),
            lastUsedAccountButton.trailingAnchor.constraint(equalTo: removeLastUsedAccountButton.leadingAnchor),
            lastUsedAccountButton.heightAnchor.constraint(lessThanOrEqualTo: privateTextField.heightAnchor),
            lastUsedAccountHeightConstraint,

            removeLastUsedAccountButton.topAnchor.constraint(equalTo: separator.bottomAnchor),
            removeLastUsedAccountButton.leadingAnchor.constraint(equalTo: lastUsedAccountButton.trailingAnchor),
            removeLastUsedAccountButton.trailingAnchor.constraint(equalTo: bottomRowView.trailingAnchor),
            removeLastUsedAccountButton.bottomAnchor.constraint(equalTo: bottomRowView.bottomAnchor),
            removeLastUsedAccountButton.widthAnchor.constraint(equalTo: removeLastUsedAccountButton.heightAnchor),
        ])

        backgroundColor = UIColor.clear
        borderLayer.lineWidth = borderWidth
        borderLayer.fillColor = UIColor.clear.cgColor
        contentView.layer.mask = contentLayerMask

        layer.insertSublayer(borderLayer, at: 0)

        updateAppearance()
        updateTextFieldEnabled()
        updateSendButtonAppearance(animated: false)
        updateKeyboardReturnKeyEnabled()

        lastUsedAccountButton.addTarget(self, action: #selector(didTapLastUsedAccount), for: .touchUpInside)

        removeLastUsedAccountButton.addTarget(self, action: #selector(didTapRemoveLastUsedAccount), for: .touchUpInside)

        addTextFieldNotificationObservers()
        addAccessibilityNotificationObservers()
        sendButton.addTarget(self, action: #selector(handleSendButton(_:)), for: .touchUpInside)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setLoginState(_ state: LoginState, animated: Bool) {
        loginState = state

        updateAppearance()
        updateTextFieldEnabled()
        updateSendButtonAppearance(animated: animated)
        updateLastUsedAccount()
    }

    func setOnReturnKey(_ onReturnKey: ((AccountInputGroupView) -> Bool)?) {
        if let onReturnKey = onReturnKey {
            privateTextField.onReturnKey = { [weak self] _ -> Bool in
                guard let self = self else { return true }

                return onReturnKey(self)
            }
        } else {
            privateTextField.onReturnKey = nil
        }
    }

    func setAccount(_ account: String) {
        privateTextField.autoformattingText = account
        updateSendButtonAppearance(animated: false)
    }

    func clearAccount() {
        privateTextField.autoformattingText = ""
        updateSendButtonAppearance(animated: false)
    }

    func setLastUsedAccount(_ accountNumber: String) {
        lastUsedAccount = accountNumber
        lastUsedAccountButton.setTitle(accountNumber, for: .normal)
        setLastUsedAccount(isExpanded: true)
    }

    // MARK: - CALayerDelegate

    override func layoutSublayers(of layer: CALayer) {
        super.layoutSublayers(of: layer)

        guard layer == self.layer else { return }

        // extend the border frame outside of the content area
        let borderFrame = layer.bounds.insetBy(dx: -borderWidth * 0.5, dy: -borderWidth * 0.5)

        // create a bezier path for border
        let borderPath = borderBezierPath(size: borderFrame.size)

        // update the background layer mask
        contentLayerMask.frame = borderFrame
        contentLayerMask.contents = backgroundMaskImage(borderPath: borderPath).cgImage

        borderLayer.path = borderPath.cgPath
        borderLayer.frame = borderFrame
    }

    // MARK: - Actions

    @objc private func textDidBeginEditing() {
        updateAppearance()
    }

    @objc private func textDidChange() {
        updateSendButtonAppearance(animated: true)
        updateKeyboardReturnKeyEnabled()
    }

    @objc private func textDidEndEditing() {
        updateAppearance()
    }

    @objc private func handleSendButton(_ sender: Any) {
        self.delegate?.accountInputGroupViewShouldAttemptLogin(self)
    }

    @objc private func didTapLastUsedAccount() {
        setAccount(lastUsedAccount)
        privateTextField.resignFirstResponder()
        setLastUsedAccount(isExpanded: false)
        self.delegate?.accountInputGroupViewShouldAttemptLogin(self)
    }

    @objc private func didTapRemoveLastUsedAccount() {
        if self.delegate?.accountInputGroupViewShouldRemoveLastUsedAccount(self) ?? false {
            clearAccount()
            setLastUsedAccount(isExpanded: false)
        }
    }

    // MARK: - Private

    private static func accountNumberFont() -> UIFont {
        if #available(iOS 13, *) {
            return UIFont.monospacedSystemFont(ofSize: 20, weight: .regular)
        } else {
            return UIFont.systemFont(ofSize: 20)
        }
    }

    private func addTextFieldNotificationObservers() {
        let notificationCenter = NotificationCenter.default

        notificationCenter.addObserver(self,
                                       selector: #selector(textDidBeginEditing),
                                       name: UITextField.textDidBeginEditingNotification,
                                       object: privateTextField)
        notificationCenter.addObserver(self,
                                       selector: #selector(textDidChange),
                                       name: UITextField.textDidChangeNotification,
                                       object: privateTextField)
        notificationCenter.addObserver(self,
                                       selector: #selector(textDidEndEditing),
                                       name: UITextField.textDidEndEditingNotification,
                                       object: privateTextField)
    }

    private func updateAppearance() {
        borderLayer.strokeColor = borderColor.cgColor
        topRowView.backgroundColor = backgroundLayerColor
        privateTextField.textColor = textColor
    }

    private func updateTextFieldEnabled() {
        switch loginState {
        case .authenticating, .success:
            privateTextField.isEnabled = false

        case .default, .failure:
            privateTextField.isEnabled = true
        }
    }

    private func updateLastUsedAccount() {
        guard !lastUsedAccount.isEmpty else {
            setLastUsedAccount(isExpanded: false)
            return
        }
        switch self.loginState {
        case .authenticating, .success:
            setLastUsedAccount(isExpanded: false)
        default:
            setLastUsedAccount(isExpanded: true)
        }
    }

    private func setLastUsedAccount(isExpanded: Bool) {
        lastUsedAccountHeightConstraint.constant = isExpanded ? 50 : 0
        lastUsedAccountButton.alpha = isExpanded ? 1 : 0
        lastUsedAccountButton.isUserInteractionEnabled = isExpanded ? true : false

        separatorHeightConstraint.constant = isExpanded ? 2 : 0
        separator.alpha = isExpanded ? 1 : 0
    }

    private func updateSendButtonAppearance(animated: Bool) {
        let actions = {
            switch self.loginState {
            case .authenticating, .success:
                // Always show the send button when voice over is running to make it discoverable
                self.sendButton.alpha = UIAccessibility.isVoiceOverRunning ? 1 : 0

                self.sendButton.isEnabled = false
                self.sendButton.backgroundColor = .lightGray

            case .default, .failure:
                let isEnabled = self.satisfiesMinimumTokenLengthRequirement

                // Always show the send button when voice over is running to make it discoverable
                if UIAccessibility.isVoiceOverRunning {
                    self.sendButton.alpha = 1
                } else {
                    self.sendButton.alpha = isEnabled ? 1 : 0
                }

                self.sendButton.isEnabled = isEnabled
                self.sendButton.backgroundColor = isEnabled ? .successColor : .lightGray
            }
        }

        if animated {
            UIView.animate(withDuration: 0.25) {
                actions()
            }
        } else {
            actions()
        }
    }

    private func updateKeyboardReturnKeyEnabled() {
        privateTextField.enableReturnKey = satisfiesMinimumTokenLengthRequirement
    }

    private func borderBezierPath(size: CGSize) -> UIBezierPath {
        let borderPath = UIBezierPath(roundedRect: CGRect(origin: .zero, size: size), cornerRadius: borderRadius)
        borderPath.lineWidth = borderWidth

        return borderPath
    }

    private func backgroundMaskImage(borderPath: UIBezierPath) -> UIImage {
        let renderer = UIGraphicsImageRenderer(bounds: borderPath.bounds)
        return renderer.image { (ctx) in
            borderPath.fill()

            // strip out any overlapping pixels between the border and the background
            borderPath.stroke(with: .clear, alpha: 0)
        }
    }

    // MARK: - Accessibility

    private func addAccessibilityNotificationObservers() {
        NotificationCenter.default.addObserver(self, selector: #selector(voiceOverStatusDidChange(_:)), name: UIAccessibility.voiceOverStatusDidChangeNotification, object: nil)
    }

    @objc private func voiceOverStatusDidChange(_ notification: Notification) {
        updateSendButtonAppearance(animated: true)
    }

}
