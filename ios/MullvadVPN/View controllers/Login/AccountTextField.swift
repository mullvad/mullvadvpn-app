//
//  AccountTextField.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class AccountTextField: CustomTextField, UITextFieldDelegate {
    enum GroupingStyle: Int {
        case full
        case lastPart

        var size: UInt8 {
            switch self {
            case .full:
                return 4
            case .lastPart:
                return 1
            }
        }
    }

    private var groupSize: GroupingStyle = .full
    private lazy var inputFormatter = InputTextFormatter(configuration: InputTextFormatter.Configuration(
        allowedInput: .numeric,
        groupSeparator: " ",
        groupSize: 4,
        maxGroups: groupSize.size
    ))

    var onReturnKey: ((AccountTextField) -> Bool)?

    init(groupingStyle: GroupingStyle = .full) {
        self.groupSize = groupingStyle
        super.init(frame: .zero)
        commonInit()
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        commonInit()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func commonInit() {
        backgroundColor = .clear
        cornerRadius = 0

        delegate = self
        pasteDelegate = inputFormatter

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(keyboardWillShow(_:)),
            name: UIWindow.keyboardWillShowNotification,
            object: nil
        )
    }

    var autoformattingText: String {
        get {
            inputFormatter.formattedString
        }
        set {
            inputFormatter.replace(with: newValue)
            inputFormatter.updateTextField(self)
        }
    }

    var parsedToken: String {
        inputFormatter.string
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
        inputFormatter.textField(
            textField,
            shouldChangeCharactersIn: range,
            replacementString: string
        )
    }

    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        onReturnKey?(self) ?? true
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
        get {
            if text?.isEmpty ?? true {
                return ""
            } else {
                return super.accessibilityValue
            }
        }
        set {
            super.accessibilityValue = newValue
        }
    }
}
