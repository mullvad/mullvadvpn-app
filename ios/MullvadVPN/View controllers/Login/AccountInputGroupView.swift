//
//  AccountInputGroupView.swift
//  MullvadVPN
//
//  Created by pronebird on 22/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import UIKit

private let animationDuration: Duration = .milliseconds(250)

final class AccountInputGroupView: UIView {
    private let minimumAccountTokenLength = 10
    private var showsLastUsedAccountRow = false

    enum Style {
        case normal, error, authenticating
    }

    let sendButton: UIButton = {
        let button = UIButton(type: .custom)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setImage(UIImage(named: "IconArrow"), for: .normal)
        button.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        button.accessibilityIdentifier = .loginTextFieldButton
        button.accessibilityLabel = NSLocalizedString(
            "ACCOUNT_INPUT_LOGIN_BUTTON_ACCESSIBILITY_LABEL",
            tableName: "AccountInput",
            value: "Log in",
            comment: ""
        )
        return button
    }()

    var textField: UITextField {
        privateTextField
    }

    var parsedToken: String {
        privateTextField.parsedToken
    }

    var satisfiesMinimumTokenLengthRequirement: Bool {
        privateTextField.parsedToken.count > minimumAccountTokenLength
    }

    var didRemoveLastUsedAccount: (() -> Void)?
    var didEnterAccount: (() -> Void)?

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
        textField.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        return textField
    }()

    private let separator: UIView = {
        let separator = UIView()
        separator.translatesAutoresizingMaskIntoConstraints = false
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
        view.accessibilityElementsHidden = true
        return view
    }()

    private let lastUsedAccountButton: UIButton = {
        let button = UIButton(type: .system)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.titleLabel?.font = accountNumberFont()
        button.setTitle(" ", for: .normal)
        button.contentHorizontalAlignment = .leading
        button.contentEdgeInsets = UIMetrics.textFieldMargins
        button.setTitleColor(UIColor.AccountTextField.NormalState.textColor, for: .normal)
        button.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        button.accessibilityLabel = NSLocalizedString(
            "LAST_USED_ACCOUNT_ACCESSIBILITY_LABEL",
            tableName: "AccountInput",
            value: "Last used account",
            comment: ""
        )
        return button
    }()

    private let removeLastUsedAccountButton: UIButton = {
        let button = UIButton(type: .custom)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setImage(UIImage(named: "IconCloseSml"), for: .normal)
        button.imageView?.tintColor = .primaryColor.withAlphaComponent(0.4)
        button.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        button.accessibilityLabel = NSLocalizedString(
            "REMOVE_LAST_USED_ACCOUNT_ACCESSIBILITY_LABEL",
            tableName: "AccountInput",
            value: "Remove last used account",
            comment: ""
        )
        return button
    }()

    let contentView: UIView = {
        let view = UIView()
        view.backgroundColor = .clear
        return view
    }()

    private(set) var loginState = LoginState.default
    private let borderRadius = CGFloat(8)
    private let borderWidth = CGFloat(2)
    private var lastUsedAccount: String?

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

    private let borderLayer = AccountInputBorderLayer()
    private let contentLayerMask = CALayer()
    private var lastUsedAccountVisibleConstraint: NSLayoutConstraint!
    private var lastUsedAccountHiddenConstraint: NSLayoutConstraint!

    // MARK: - View lifecycle

    override init(frame: CGRect) {
        super.init(frame: frame)
        configUI()
        setAppearance()
        addActions()
        updateAppearance()
        updateTextFieldEnabled()
        updateSendButtonAppearance(animated: false)
        updateKeyboardReturnKeyEnabled()
        addTextFieldNotificationObservers()
        addAccessibilityNotificationObservers()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func configUI() {
        addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview(.all().excluding(.bottom))
        }

        contentView.addConstrainedSubviews([topRowView, separator, bottomRowView]) {
            topRowView.pinEdgesToSuperview(.all().excluding(.bottom))
            topRowView.bottomAnchor.constraint(equalTo: separator.topAnchor)

            separator.pinEdgesToSuperview(.all().excluding([.bottom, .top]))
            separator.topAnchor.constraint(equalTo: topRowView.bottomAnchor)
            separator.heightAnchor.constraint(equalToConstant: borderWidth)

            bottomRowView.topAnchor.constraint(equalTo: separator.bottomAnchor)
            bottomRowView.pinEdgesToSuperview(.all().excluding(.top))
        }

        topRowView.addConstrainedSubviews([privateTextField, sendButton]) {
            privateTextField.trailingAnchor.constraint(equalTo: sendButton.leadingAnchor)
            privateTextField.pinEdgesToSuperview(.all().excluding(.trailing))

            sendButton.pinEdgesToSuperview(.all().excluding(.leading))
            sendButton.widthAnchor.constraint(equalTo: sendButton.heightAnchor)
        }

        bottomRowView.addConstrainedSubviews([lastUsedAccountButton, removeLastUsedAccountButton]) {
            lastUsedAccountButton.pinEdgesToSuperview(.all().excluding(.trailing))
            lastUsedAccountButton.trailingAnchor.constraint(equalTo: removeLastUsedAccountButton.leadingAnchor)

            removeLastUsedAccountButton.pinEdgesToSuperview(.all().excluding(.leading))
            removeLastUsedAccountButton.widthAnchor.constraint(equalTo: sendButton.widthAnchor)
        }

        lastUsedAccountVisibleConstraint = heightAnchor.constraint(equalTo: contentView.heightAnchor)
        lastUsedAccountHiddenConstraint = heightAnchor.constraint(equalTo: topRowView.heightAnchor)
        lastUsedAccountHiddenConstraint.isActive = true
    }

    private func setAppearance() {
        backgroundColor = UIColor.clear
        borderLayer.lineWidth = borderWidth
        borderLayer.fillColor = UIColor.clear.cgColor
        contentView.layer.mask = contentLayerMask
        layer.insertSublayer(borderLayer, at: 0)
    }

    private func addActions() {
        lastUsedAccountButton.addTarget(
            self,
            action: #selector(didTapLastUsedAccount),
            for: .touchUpInside
        )

        removeLastUsedAccountButton.addTarget(
            self,
            action: #selector(didTapRemoveLastUsedAccount),
            for: .touchUpInside
        )

        sendButton.addTarget(self, action: #selector(handleSendButton(_:)), for: .touchUpInside)
    }

    func setLoginState(_ state: LoginState, animated: Bool) {
        loginState = state

        updateAppearance()
        updateTextFieldEnabled()
        updateSendButtonAppearance(animated: animated)
        updateLastUsedAccountConstraints(animated: animated)
    }

    func setOnReturnKey(_ onReturnKey: ((AccountInputGroupView) -> Bool)?) {
        if let onReturnKey {
            privateTextField.onReturnKey = { [weak self] _ -> Bool in
                guard let self else { return true }

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

    func setLastUsedAccount(_ accountNumber: String?, animated: Bool) {
        if let accountNumber {
            let formattedNumber = accountNumber.formattedAccountNumber

            lastUsedAccountButton.accessibilityAttributedValue = NSAttributedString(
                string: accountNumber,
                attributes: [.accessibilitySpeechSpellOut: true]
            )

            UIView.performWithoutAnimation {
                self.lastUsedAccountButton.setTitle(formattedNumber, for: .normal)
                self.lastUsedAccountButton.layoutIfNeeded()
            }
        }

        lastUsedAccount = accountNumber
        updateLastUsedAccountConstraints(animated: animated)
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
        didEnterAccount?()
    }

    @objc private func didTapLastUsedAccount() {
        guard let lastUsedAccount else { return }

        setAccount(lastUsedAccount)
        privateTextField.resignFirstResponder()

        updateLastUsedAccountConstraints(animated: true)
        didEnterAccount?()
    }

    @objc private func didTapRemoveLastUsedAccount() {
        didRemoveLastUsedAccount?()
        setLastUsedAccount(nil, animated: true)
    }

    // MARK: - Private

    private static func accountNumberFont() -> UIFont {
        UIFont.monospacedSystemFont(ofSize: 20, weight: .regular)
    }

    private func addTextFieldNotificationObservers() {
        let notificationCenter = NotificationCenter.default

        notificationCenter.addObserver(
            self,
            selector: #selector(textDidBeginEditing),
            name: UITextField.textDidBeginEditingNotification,
            object: privateTextField
        )
        notificationCenter.addObserver(
            self,
            selector: #selector(textDidChange),
            name: UITextField.textDidChangeNotification,
            object: privateTextField
        )
        notificationCenter.addObserver(
            self,
            selector: #selector(textDidEndEditing),
            name: UITextField.textDidEndEditingNotification,
            object: privateTextField
        )
    }

    private func updateAppearance() {
        borderLayer.strokeColor = borderColor.cgColor
        separator.backgroundColor = borderColor
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

    private func shouldShowLastUsedAccountRow() -> Bool {
        guard lastUsedAccount != nil else {
            return false
        }

        switch loginState {
        case .authenticating, .success:
            return false
        case .default, .failure:
            return true
        }
    }

    private func updateLastUsedAccountConstraints(animated: Bool) {
        let shouldShow = shouldShowLastUsedAccountRow()

        guard showsLastUsedAccountRow != shouldShow else {
            return
        }

        let actions = {
            let constraintToDeactivate: NSLayoutConstraint
            let constraintToActivate: NSLayoutConstraint

            if shouldShow {
                constraintToActivate = self.lastUsedAccountVisibleConstraint
                constraintToDeactivate = self.lastUsedAccountHiddenConstraint
            } else {
                constraintToActivate = self.lastUsedAccountHiddenConstraint
                constraintToDeactivate = self.lastUsedAccountVisibleConstraint
            }

            constraintToDeactivate.isActive = false
            constraintToActivate.isActive = true
        }

        if animated {
            actions()
            UIView.animate(withDuration: animationDuration.timeInterval) {
                self.layoutIfNeeded()
            }
        } else {
            actions()
            setNeedsLayout()
        }

        showsLastUsedAccountRow = shouldShow

        bottomRowView.accessibilityElementsHidden = !shouldShow

        if lastUsedAccountButton.accessibilityElementIsFocused() ||
            removeLastUsedAccountButton.accessibilityElementIsFocused() {
            UIAccessibility.post(notification: .layoutChanged, argument: textField)
        }
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
            UIView.animate(withDuration: animationDuration.timeInterval) {
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
        let borderPath = UIBezierPath(
            roundedRect: CGRect(origin: .zero, size: size),
            cornerRadius: borderRadius
        )
        borderPath.lineWidth = borderWidth

        return borderPath
    }

    private func backgroundMaskImage(borderPath: UIBezierPath) -> UIImage {
        let renderer = UIGraphicsImageRenderer(bounds: borderPath.bounds)
        return renderer.image { _ in
            borderPath.fill()
            // strip out any overlapping pixels between the border and the background
            borderPath.stroke(with: .clear, alpha: 0)
        }
    }

    // MARK: - Accessibility

    private func addAccessibilityNotificationObservers() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(voiceOverStatusDidChange(_:)),
            name: UIAccessibility.voiceOverStatusDidChangeNotification,
            object: nil
        )
    }

    @objc private func voiceOverStatusDidChange(_ notification: Notification) {
        updateSendButtonAppearance(animated: true)
    }
}

private class AccountInputBorderLayer: CAShapeLayer {
    override class func defaultAction(forKey event: String) -> CAAction? {
        if event == "path" {
            let action = CABasicAnimation(keyPath: event)
            action.duration = animationDuration.timeInterval
            action.timingFunction = CAMediaTimingFunction(name: .easeInEaseOut)

            return action
        }
        return super.defaultAction(forKey: event)
    }

    // swiftlint:disable:next file_length
}
