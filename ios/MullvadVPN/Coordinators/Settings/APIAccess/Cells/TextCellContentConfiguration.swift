//
//  TextCellContentConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 09/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content configuration presenting a label and text field.
struct TextCellContentConfiguration: UIContentConfiguration, Equatable {
    /// The text label.
    var text: String?

    /// The input field text.
    var inputText: String?

    /// The text input filter that can be used to prevent user from entering illegal characters.
    var inputFilter: TextInputFilter = .allowAll

    /// The maximum input length.
    var maxLength: Int?

    /// The text field placeholder.
    var placeholder: String?

    /// The editing events configuration.
    var editingEvents = EditingEvents()

    /// The text properties confgiuration.
    var textProperties = TextProperties()

    /// The text field properties configuration.
    var textFieldProperties = TextFieldProperties()

    /// The content view layout margins.
    var directionalLayoutMargins: NSDirectionalEdgeInsets = UIMetrics.SettingsCell.apiAccessInsetLayoutMargins

    func makeContentView() -> UIView & UIContentView {
        return TextCellContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}

extension TextCellContentConfiguration {
    /// The text label properties.
    struct TextProperties: Equatable {
        var font = UIFont.systemFont(ofSize: 17)
        var color = UIColor.Cell.titleTextColor
    }

    /// The text input filter.
    enum TextInputFilter: Equatable {
        /// Allow all input.
        case allowAll

        /// Allow digits only.
        case digitsOnly
    }

    /// Editing events configuration assigned on the text field.
    struct EditingEvents: Equatable {
        /// The action invoked on text field input change.
        var onChange: UIAction?
        /// The action invoked when text field begins editing.
        var onBegin: UIAction?
        /// The action invoked when text field ends editing.
        var onEnd: UIAction?
        /// The action invoked on the touch ending the editing session. (i.e. the return key)
        var onEndOnExit: UIAction?
    }

    /// Text field configuration.
    struct TextFieldProperties: Equatable {
        /// Text font.
        var font = UIFont.systemFont(ofSize: 17)
        /// Text color.
        var textColor = UIColor.Cell.textFieldTextColor

        /// Placeholder color.
        var placeholderColor = UIColor.Cell.textFieldPlaceholderColor

        /// Automatically resign keyboard on return key.
        var resignOnReturn = false

        /// Text content type.
        var textContentType: UITextContentType?

        /// Keyboard type.
        var keyboardType: UIKeyboardType = .default

        /// Return key type.
        var returnKey: UIReturnKeyType = .default

        /// Indicates whether the text input should be obscured.
        /// Set to `true` for password entry.
        var isSecureTextEntry = false

        /// Autocorrection type.
        var autocorrectionType: UITextAutocorrectionType = .default

        /// Autocapitalization type.
        var autocapitalizationType: UITextAutocapitalizationType = .sentences

        /// Spellchecking type.
        var spellCheckingType: UITextSpellCheckingType = .default

        var smartInsertDeleteType: UITextSmartInsertDeleteType = .default
        var smartDashesType: UITextSmartDashesType = .default
        var smartQuotesType: UITextSmartQuotesType = .default

        /// An option set describing a set of text field features to enable or disable in bulk.
        struct Features: OptionSet {
            /// Autocorrection.
            static let autoCorrect = Features(rawValue: 1 << 1)
            /// Spellcheck.
            static let spellCheck = Features(rawValue: 1 << 2)
            /// Autocapitalization.
            static let autoCapitalization = Features(rawValue: 1 << 3)
            /// Smart features such as automatic hyphenation or insertion of a space at the end of word etc.
            static let smart = Features(rawValue: 1 << 4)
            /// All of the above.
            static let all = Features([.autoCorrect, .spellCheck, .autoCapitalization, .smart])

            let rawValue: Int
        }

        /// Produce text field configuration with the given text field features disabled.
        /// - Parameter features: the text field features to disable.
        /// - Returns: new text field configuration.
        func disabling(features: Features) -> TextFieldProperties {
            var mutableProperties = self
            mutableProperties.disable(features: features)
            return mutableProperties
        }

        /// Disable a set of text field features mutating the current configuration in-place.
        /// - Parameter features: the text field features to disable.
        mutating func disable(features: Features) {
            if features.contains(.autoCorrect) {
                autocorrectionType = .no
            }
            if features.contains(.spellCheck) {
                spellCheckingType = .no
            }
            if features.contains(.autoCapitalization) {
                autocapitalizationType = .none
            }
            if features.contains(.smart) {
                smartInsertDeleteType = .no
                smartDashesType = .no
                smartQuotesType = .no
            }
        }
    }
}
