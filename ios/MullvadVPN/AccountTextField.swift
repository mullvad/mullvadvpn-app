//
//  AccountTextField.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class AccountTextField: CustomTextField, UITextFieldDelegate, UITextPasteDelegate {
    /// The size of groups of digits.
    static let groupSize = 4

    /// Spacing between groups in points.
    /// Automatically updated using current font.
    private var groupSpacing: CGFloat = 8

    /// Adjust caret by one whitespace when it's at the end of document, unless the given character
    /// limit reached.
    static let caretTrailingSpaceAtEndCharacterLimit = 16

    var onReturnKey: ((AccountTextField) -> Bool)?

    override var font: UIFont? {
        didSet {
            updateGroupSpacing()
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)

        backgroundColor = .clear
        cornerRadius = 0

        delegate = self
        pasteDelegate = self

        addTarget(self, action: #selector(textDidChange), for: .editingChanged)

        updateGroupSpacing()

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
            let string = newValue.filter(Self.isDigit)
            attributedText = styleInput(string)
        }
        get {
            return (text ?? "").filter(Self.isDigit)
        }
    }

    var enableReturnKey: Bool = true {
        didSet {
            updateKeyboardReturnKey()
        }
    }

    private class func isDigit(_ ch: Character) -> Bool {
        switch ch {
        case "0"..."9":
            return true
        default:
            return false
        }
    }

    // MARK: - Actions

    override func canPerformAction(_ action: Selector, withSender sender: Any?) -> Bool {
         if #available(iOS 15.0, *) {
             if action == #selector(captureTextFromCamera(_:)) {
                 return false
             }
         }
         return super.canPerformAction(action, withSender: sender)
     }

    @objc func textDidChange() {
        let selection = selectedTextRange
        attributedText = text.map { styleInput($0) }
        selectedTextRange = selection
    }

    // MARK: - Input styling

    private func styleInput(_ string: String) -> NSAttributedString {
        let attributedString = NSMutableAttributedString(string: string)

        for i in 0 ..< string.count {
            if i > 0 && i % Self.groupSize == 0 {
                let start = string.index(string.startIndex, offsetBy: i - 1)
                let nsRange = NSRange(start ... start, in: string)

                attributedString.addAttribute(.kern, value: groupSpacing, range: nsRange)
            }
        }

        return attributedString
    }

    private func updateGroupSpacing() {
        let measurementFont = font ?? UIFont.systemFont(ofSize: UIFont.systemFontSize)
        let size = " ".size(withAttributes: [.font: measurementFont])

        groupSpacing = size.width
    }

    // MARK: - UITextPasteDelegate

    func textPasteConfigurationSupporting(_ textPasteConfigurationSupporting: UITextPasteConfigurationSupporting, transform item: UITextPasteItem) {
        _ = item.itemProvider.loadObject(ofClass: String.self) { str, error in
            if let str = str {
                let parsedString = str.filter(Self.isDigit)
                item.setResult(string: parsedString)
            } else {
                item.setNoResult()
            }
        }
    }

    func textPasteConfigurationSupporting(_ textPasteConfigurationSupporting: UITextPasteConfigurationSupporting, performPasteOf attributedString: NSAttributedString, to textRange: UITextRange) -> UITextRange {
        attributedText = styleInput(attributedString.string)

        // FIXME: triggers extra pass via `textDidChange()`.
        sendActions(for: .editingChanged)

        NotificationCenter.default.post(name: UITextField.textDidChangeNotification, object: self)

        return self.textRange(from: endOfDocument, to: endOfDocument)!
    }

    // MARK: - UITextFieldDelegate

    func textField(_ textField: UITextField, shouldChangeCharactersIn range: NSRange, replacementString string: String) -> Bool {
        return string.allSatisfy(Self.isDigit)
    }

    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        return onReturnKey?(self) ?? true
    }

    // MARK: - UITextInput

    override func caretRect(for position: UITextPosition) -> CGRect {
        var caretRect = super.caretRect(for: position)

        let offset = offset(from: beginningOfDocument, to: position)

        if offset > 0 && offset % Self.groupSize == 0 {
            // Compensate kerning.
            var spacing: CGFloat = .zero

            if position == endOfDocument {
                let textLength = text?.count ?? 0

                if textLength < Self.caretTrailingSpaceAtEndCharacterLimit {
                    spacing = groupSpacing
                }
            } else {
                spacing = -groupSpacing
            }

            caretRect.origin.x += spacing
        }

        return caretRect
    }

    // MARK: - Notifications

    @objc private func keyboardWillShow(_ notification: Notification) {
        if self.isFirstResponder {
            updateKeyboardReturnKey()
        }
    }

    // MARK: - Keyboard

    private func updateKeyboardReturnKey() {
        setEnableKeyboardReturnKey(enableReturnKey)
    }

    private func setEnableKeyboardReturnKey(_ enableReturnKey: Bool) {
        let selector = NSSelectorFromString("setReturnKeyEnabled:")
        if let inputDelegate = self.inputDelegate as? NSObject, inputDelegate.responds(to: selector) {
            inputDelegate.setValue(enableReturnKey, forKey: "returnKeyEnabled")
        }
    }

    // MARK: - Accessibility

    override var accessibilityValue: String? {
        set {
            super.accessibilityValue = newValue
        }
        get {
            if self.text?.isEmpty ?? true {
                return ""
            } else {
                return super.accessibilityValue
            }
        }
    }

}
