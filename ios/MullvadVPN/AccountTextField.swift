//
//  AccountTextField.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class AccountTextField: UITextField, UITextFieldDelegate {

    private let input = AccountTokenInput()

    var onReturnKey: ((AccountTextField) -> Bool)?

    override init(frame: CGRect) {
        super.init(frame: frame)
        setup()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
        setup()
    }

    private func setup() {
        backgroundColor = UIColor.clear

        delegate = self
        pasteDelegate = input

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(keyboardWillShow(_:)),
            name: UIWindow.keyboardWillShowNotification,
            object: nil
        )
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

    var enableReturnKey: Bool = true {
        didSet {
            updateKeyboardReturnKey()
        }
    }

    override func textRect(forBounds bounds: CGRect) -> CGRect {
        return bounds.insetBy(dx: 14, dy: 12)
    }

    override func editingRect(forBounds bounds: CGRect) -> CGRect {
        return textRect(forBounds: bounds)
    }

    // MARK: - UITextFieldDelegate

    func textField(_ textField: UITextField, shouldChangeCharactersIn range: NSRange, replacementString string: String) -> Bool {
        return input.textField(textField, shouldChangeCharactersIn: range, replacementString: string)
    }

    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        return onReturnKey?(self) ?? true
    }

    // MARK: - Keyboard notifications

    @objc private func keyboardWillShow(_ notification: Notification) {
        if self.isFirstResponder {
            updateKeyboardReturnKey()
        }
    }

    private func updateKeyboardReturnKey() {
        setEnableKeyboardReturnKey(enableReturnKey)
    }

    private func setEnableKeyboardReturnKey(_ enableReturnKey: Bool) {
        let selector = NSSelectorFromString("setReturnKeyEnabled:")
        if let inputDelegate = self.inputDelegate as? NSObject, inputDelegate.responds(to: selector) {
            inputDelegate.setValue(enableReturnKey, forKey: "returnKeyEnabled")
        }
    }

}
