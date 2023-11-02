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
    var text: String?
    var inputText: String?
    var inputFilter: TextInputFilter = .allowAll
    var placeholder: String?

    var editingEvents = EditingEvents()

    var textProperties = TextProperties()
    var textFieldProperties = TextFieldProperties()

    var directionalLayoutMargins: NSDirectionalEdgeInsets = UIMetrics.SettingsCell.insetLayoutMargins

    func makeContentView() -> UIView & UIContentView {
        return TextCellContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}

extension TextCellContentConfiguration {
    struct TextProperties: Equatable {
        var font = UIFont.systemFont(ofSize: 17)
        var color = UIColor.Cell.titleTextColor
    }

    enum TextInputFilter: Equatable {
        case allowAll
        case digitsOnly
    }

    struct EditingEvents: Equatable {
        var onChange: UIAction?
        var onBegin: UIAction?
        var onEnd: UIAction?
        var onEndOnExit: UIAction?
    }

    struct TextFieldProperties: Equatable {
        var font = UIFont.systemFont(ofSize: 17)
        var textColor = UIColor.Cell.textFieldTextColor
        var placeholderColor = UIColor.Cell.textFieldPlaceholderColor

        /// Automatically resign keyboard on return key.
        var resignOnReturn = false

        var textContentType: UITextContentType?
        var keyboardType: UIKeyboardType = .default
        var returnKey: UIReturnKeyType = .default
        var isSecureTextEntry = false

        var autocorrectionType: UITextAutocorrectionType = .default
        var autocapitalizationType: UITextAutocapitalizationType = .sentences
        var spellCheckingType: UITextSpellCheckingType = .default

        var smartInsertDeleteType: UITextSmartInsertDeleteType = .default
        var smartDashesType: UITextSmartDashesType = .default
        var smartQuotesType: UITextSmartQuotesType = .default

        struct Features: OptionSet {
            static let autoCorrect = Features(rawValue: 1 << 1)
            static let spellCheck = Features(rawValue: 1 << 2)
            static let autoCapitalization = Features(rawValue: 1 << 3)
            static let smart = Features(rawValue: 1 << 4)

            static let all = Features([.autoCorrect, .spellCheck, .autoCapitalization, .smart])

            let rawValue: Int
        }

        func disabling(features: Features) -> TextFieldProperties {
            var mutableProperties = self
            mutableProperties.disable(features: features)
            return mutableProperties
        }

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
