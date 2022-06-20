//
//  AccountTextField.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class AccountTextField: CustomTextField, UITextFieldDelegate, UITextPasteDelegate {
    /// The size of a single group of digits.
    static let digitGroupSize = 4

    /// Spacing between groups in points.
    /// Automatically updated using current font.
    private var digitGroupSpacing: CGFloat = 8

    /// Max number of groups of digits expected during input.
    static let maxExpectedDigitGroupsCount = 4

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
            if Self.isEndOfDigitGroup(at: i + 1) {
                let start = string.index(string.startIndex, offsetBy: i)
                let nsRange = NSRange(start ... start, in: string)

                attributedString.addAttribute(.kern, value: digitGroupSpacing, range: nsRange)
            }
        }

        return attributedString
    }

    private func updateGroupSpacing() {
        let measurementFont = font ?? UIFont.systemFont(ofSize: UIFont.systemFontSize)
        let size = " ".size(withAttributes: [.font: measurementFont])

        digitGroupSpacing = size.width
    }

    private func maxDigitGroupsExceeded() -> Bool {
        let length = text?.count ?? 0
        let groupCount = Float(length) / Float(Self.digitGroupSize)
        let roundedGroupCount = Int(groupCount.rounded(.up))

        return roundedGroupCount >= Self.maxExpectedDigitGroupsCount
    }

    private class func isEndOfDigitGroup(at characterIndex: Int) -> Bool {
        return characterIndex > 0 && (characterIndex % digitGroupSize) == 0
    }

    private class func isDigit(_ ch: Character) -> Bool {
        return ("0"..."9").contains(ch)
    }

    // MARK: - UITextPasteDelegate

    func textPasteConfigurationSupporting(
        _ textPasteConfigurationSupporting: UITextPasteConfigurationSupporting,
        transform item: UITextPasteItem
    )
    {
        _ = item.itemProvider.loadObject(ofClass: String.self) { string, error in
            if let string = string {
                let parsedString = string.filter(Self.isDigit)
                item.setResult(string: parsedString)
            } else {
                item.setNoResult()
            }
        }
    }

    func textPasteConfigurationSupporting(
        _ textPasteConfigurationSupporting: UITextPasteConfigurationSupporting,
        performPasteOf attributedString: NSAttributedString,
        to textRange: UITextRange
    ) -> UITextRange
    {
        attributedText = styleInput(attributedString.string)

        // FIXME: triggers extra pass via `textDidChange()`.
        sendActions(for: .editingChanged)

        NotificationCenter.default.post(name: UITextField.textDidChangeNotification, object: self)

        return self.textRange(from: endOfDocument, to: endOfDocument)!
    }

    // MARK: - UITextFieldDelegate

    func textField(
        _ textField: UITextField,
        shouldChangeCharactersIn range: NSRange,
        replacementString string: String
    ) -> Bool {
        return string.allSatisfy(Self.isDigit)
    }

    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        return onReturnKey?(self) ?? true
    }

    // MARK: - UITextInput

    override func caretRect(for position: UITextPosition) -> CGRect {
        var caretRect = super.caretRect(for: position)
        let offset = offset(from: beginningOfDocument, to: position)

        if position == endOfDocument && Self.isEndOfDigitGroup(at: offset) {
            /// Prevent moving caret to the next group of digits once the number of expected groups
            /// is reached.
            if #available(iOS 15, *) {
                // TextKit2 does not include kerning to the trailing character.
                if !maxDigitGroupsExceeded() {
                    caretRect.origin.x += digitGroupSpacing
                }
            } else {
                if maxDigitGroupsExceeded() {
                    caretRect.origin.x -= digitGroupSpacing
                }
            }
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
