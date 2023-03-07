//
//  AccountTextField.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class AccountTextField: CustomTextField, UITextFieldDelegate {
    private let input = AccountTokenInput()

    var onReturnKey: ((AccountTextField) -> Bool)?

    override init(frame: CGRect) {
        super.init(frame: frame)

        backgroundColor = .clear
        cornerRadius = 0

        delegate = self
        pasteDelegate = input

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(keyboardWillShow(_:)),
            name: UIWindow.keyboardWillShowNotification,
            object: nil
        )
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    var autoformattingText: String {
        set {
            input.replace(with: newValue)
            input.updateTextField(self)
        }
        get {
            input.formattedString
        }
    }

    var parsedToken: String {
        return input.parsedString
    }

    var enableReturnKey = true {
        didSet {
            updateKeyboardReturnKey()
        }
    }

    override func canPerformAction(_ action: Selector, withSender sender: Any?) -> Bool {
        if #available(iOS 15.0, *) {
            if action == #selector(captureTextFromCamera(_:)) {
                return false
            }
        }
        return super.canPerformAction(action, withSender: sender)
    }

    // MARK: - UITextFieldDelegate

    func textField(
        _ textField: UITextField,
        shouldChangeCharactersIn range: NSRange,
        replacementString string: String
    ) -> Bool {
        return input.textField(
            textField,
            shouldChangeCharactersIn: range,
            replacementString: string
        )
    }

    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        return onReturnKey?(self) ?? true
    }

    // MARK: - Notifications

    @objc private func keyboardWillShow(_ notification: Notification) {
        if isFirstResponder {
            updateKeyboardReturnKey()
        }
    }

    // MARK: - Keyboard

    private func updateKeyboardReturnKey() {
        setEnableKeyboardReturnKey(enableReturnKey)
    }

    private func setEnableKeyboardReturnKey(_ enableReturnKey: Bool) {
        let selector = NSSelectorFromString("setReturnKeyEnabled:")
        if let inputDelegate = inputDelegate as? NSObject, inputDelegate.responds(to: selector) {
            inputDelegate.setValue(enableReturnKey, forKey: "returnKeyEnabled")
        }
    }

    // MARK: - Accessibility

    override var accessibilityValue: String? {
        set {
            super.accessibilityValue = newValue
        }
        get {
            if text?.isEmpty ?? true {
                return ""
            } else {
                return super.accessibilityValue
            }
        }
    }
}
