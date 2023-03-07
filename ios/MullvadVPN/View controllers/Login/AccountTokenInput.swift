//
//  AccountTokenInput.swift
//  MullvadVPN
//
//  Created by pronebird on 08/04/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

/// A class describing the account token input and caret management.
/// Suitable to be used with `UITextField`.
class AccountTokenInput: NSObject {
    /// The group separator character
    static let groupSeparator: Character = " "

    /// The character size of each group of digits
    static let groupSize = 4

    /// Parsed account token string
    private(set) var parsedString = ""

    /// Formatted string
    private(set) var formattedString = ""

    // Computed caret position
    private(set) var caretPosition = 0

    init(string: String = "") {
        super.init()

        replace(with: string)
    }

    /// Replace the currently held value with the given string
    func replace(with replacementString: String) {
        let stringRange = formattedString.startIndex ..< formattedString.endIndex

        replaceCharacters(
            in: stringRange,
            replacementString: replacementString,
            emptySelection: false
        )
    }

    /// Replace characters in range maintaining the caret position
    ///
    /// - Parameter range: a range within a string to replace
    /// - Parameter replacementString: a string to replace the characters in the given range
    /// - Parameter emptySelection: a hint to indicate if the text field selection is empty.
    ///                             This is normally the default state unless a text range is
    ///                             selected.
    ///
    func replaceCharacters(
        in range: Range<String.Index>,
        replacementString: String,
        emptySelection: Bool
    ) {
        var stringRange = range

        // Since removing separator alone makes no sense, this computation extends the string range
        // to include the digit preceding a separator.
        if replacementString.isEmpty, emptySelection, !formattedString.isEmpty {
            let precedingDigitIndex = formattedString
                .prefix(through: stringRange.lowerBound)
                .lastIndex { Self.isDigit($0) } ?? formattedString.startIndex

            stringRange = precedingDigitIndex ..< stringRange.upperBound
        }

        // Replace the given range within a formatted string
        let newString = formattedString.replacingCharacters(
            in: stringRange,
            with: replacementString
        )

        // Number of digits within a string
        var numDigits = 0

        // Insertion location within the input string
        let insertionLocation = formattedString.distance(
            from: formattedString.startIndex,
            to: stringRange.lowerBound
        )

        // Original caret location based on insertion location + number of characters added
        let originalCaretPosition = insertionLocation + replacementString.count

        // Computed caret location that will be modified during the loop
        var newCaretPosition = originalCaretPosition

        // New re-parsed and re-formatted strings
        var reparsedString = ""
        var reformattedString = ""

        for (index, element) in newString.enumerated() {
            // Skip disallowed characters
            if !Self.isDigit(element) {
                // Adjust the caret position for characters removed before the insertion location
                if originalCaretPosition > index {
                    newCaretPosition -= 1
                }
                continue
            }

            // Add separator between the groups of digits
            if numDigits > 0, numDigits % Self.groupSize == 0 {
                reformattedString.append(Self.groupSeparator)

                if originalCaretPosition > index {
                    // Adjust the caret position to account for separators added before the
                    // insertion location
                    newCaretPosition += 1
                }
            }

            reformattedString.append(element)
            reparsedString.append(element)
            numDigits += 1
        }

        caretPosition = newCaretPosition
        formattedString = reformattedString
        parsedString = reparsedString
    }

    private class func isDigit(_ character: Character) -> Bool {
        switch character {
        case "0" ... "9":
            return true
        default:
            return false
        }
    }
}

extension AccountTokenInput: UITextFieldDelegate, UITextPasteDelegate {
    /// Update the text and caret position in the given text field
    func updateTextField(_ textField: UITextField) {
        updateTextField(textField, notifyDelegate: false)
    }

    // MARK: - UITextFieldDelegate

    func textField(
        _ textField: UITextField,
        shouldChangeCharactersIn range: NSRange,
        replacementString string: String
    ) -> Bool {
        let emptySelection = textField.selectedTextRange?.isEmpty ?? false
        let stringRange = Range(range, in: formattedString)!

        replaceCharacters(
            in: stringRange,
            replacementString: string,
            emptySelection: emptySelection
        )

        updateTextField(textField, notifyDelegate: true)

        return false
    }

    // MARK: - UITextPasteDelegate

    func textPasteConfigurationSupporting(
        _ textPasteConfigurationSupporting: UITextPasteConfigurationSupporting,
        performPasteOf attributedString: NSAttributedString,
        to textRange: UITextRange
    ) -> UITextRange {
        guard let textField = textPasteConfigurationSupporting as? UITextField else {
            return textRange
        }

        let location = textField.offset(from: textField.beginningOfDocument, to: textRange.start)
        let length = textField.offset(from: textRange.start, to: textRange.end)
        let nsRange = NSRange(location: location, length: length)

        let stringRange = Range(nsRange, in: formattedString)!

        replaceCharacters(
            in: stringRange,
            replacementString: attributedString.string,
            emptySelection: textRange.isEmpty
        )
        updateTextField(textField, notifyDelegate: true)

        return caretTextRange(in: textField)!
    }

    // MARK: - Private

    /// A caret position as utf-16 offset compatible for use with `NSString` and `UITextField`
    private var caretPositionUtf16: Int {
        let startIndex = formattedString.startIndex
        let endIndex = formattedString.index(startIndex, offsetBy: caretPosition)

        return formattedString.utf16.distance(from: startIndex, to: endIndex)
    }

    /// Convert the computed caret position to an empty `UITextRange` within the given text field
    private func caretTextRange(in textField: UITextField) -> UITextRange? {
        guard let position = textField.position(
            from: textField.beginningOfDocument,
            offset: caretPositionUtf16
        ) else { return nil }

        return textField.textRange(from: position, to: position)
    }

    /// A helper to update the text and caret in the given text field, and optionally post
    /// `UITextField.textDidChange` notification
    private func updateTextField(_ textField: UITextField, notifyDelegate: Bool) {
        textField.text = formattedString
        textField.selectedTextRange = caretTextRange(in: textField)

        if notifyDelegate {
            Self.notifyTextDidChange(in: textField)
        }
    }

    /// Post `UITextField.textDidChange` notification
    private class func notifyTextDidChange(in textField: UITextField) {
        NotificationCenter.default.post(
            name: UITextField.textDidChangeNotification,
            object: textField
        )
    }
}
