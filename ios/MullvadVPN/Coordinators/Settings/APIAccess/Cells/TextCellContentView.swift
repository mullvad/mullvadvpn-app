//
//  TextCellContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 09/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content view presenting a label and text field.
class TextCellContentView: UIView, UIContentView, UIGestureRecognizerDelegate {
    private var textLabel = UILabel()
    private var textField = CustomTextField()

    var configuration: UIContentConfiguration {
        get {
            actualConfiguration
        }
        set {
            guard let newConfiguration = newValue as? TextCellContentConfiguration,
                  actualConfiguration != newConfiguration else { return }

            let previousConfiguration = actualConfiguration
            actualConfiguration = newConfiguration

            configureSubviews(previousConfiguration: previousConfiguration)
        }
    }

    private var actualConfiguration: TextCellContentConfiguration

    func supports(_ configuration: UIContentConfiguration) -> Bool {
        configuration is TextCellContentConfiguration
    }

    init(configuration: TextCellContentConfiguration) {
        actualConfiguration = configuration

        super.init(frame: CGRect(x: 0, y: 0, width: 100, height: 44))

        configureSubviews()
        addSubviews()
        addTapGestureRecognizer()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func configureSubviews(previousConfiguration: TextCellContentConfiguration? = nil) {
        guard actualConfiguration != previousConfiguration else { return }

        configureTextLabel()
        configureTextField()
        configureLayoutMargins()
        configureActions(previousConfiguration: previousConfiguration)
    }

    private func configureActions(previousConfiguration: TextCellContentConfiguration? = nil) {
        previousConfiguration?.editingEvents.unregister(from: textField)
        actualConfiguration.editingEvents.register(in: textField)
    }

    private func configureLayoutMargins() {
        directionalLayoutMargins = actualConfiguration.directionalLayoutMargins
    }

    private func configureTextLabel() {
        let textProperties = actualConfiguration.textProperties

        textLabel.font = textProperties.font
        textLabel.textColor = textProperties.color

        textLabel.text = actualConfiguration.text
    }

    private func configureTextField() {
        textField.text = actualConfiguration.inputText
        textField.placeholder = actualConfiguration.placeholder
        textField.delegate = self

        actualConfiguration.textFieldProperties.apply(to: textField)
    }

    private func addSubviews() {
        textField.setContentHuggingPriority(.defaultLow, for: .horizontal)
        textField.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)

        textLabel.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        textLabel.setContentCompressionResistancePriority(.defaultHigh + 1, for: .horizontal)

        addConstrainedSubviews([textLabel, textField]) {
            textField.pinEdgesToSuperviewMargins(.all().excluding(.leading))
            textLabel.pinEdgesToSuperviewMargins(.all().excluding(.trailing))
            textField.leadingAnchor.constraint(equalToSystemSpacingAfter: textLabel.trailingAnchor, multiplier: 1)
        }
    }

    // MARK: - Gesture recognition

    /// Add tap recognizer that activates the text field on tap anywhere within the content view.
    private func addTapGestureRecognizer() {
        let tapGestureRecognizer = UITapGestureRecognizer(target: self, action: #selector(handleTap(_:)))
        tapGestureRecognizer.delegate = self
        addGestureRecognizer(tapGestureRecognizer)
    }

    @objc private func handleTap(_ gestureRecognizer: UIGestureRecognizer) {
        if gestureRecognizer.state == .ended {
            textField.selectedTextRange = textField.textRange(
                from: textField.endOfDocument,
                to: textField.endOfDocument
            )
            textField.becomeFirstResponder()
        }
    }

    override func gestureRecognizerShouldBegin(_ gestureRecognizer: UIGestureRecognizer) -> Bool {
        // Allow our tap recognizer to evaluate only when the text field is not the first responder yet.
        super.gestureRecognizerShouldBegin(gestureRecognizer) && !textField.isFirstResponder
    }

    func gestureRecognizer(
        _ gestureRecognizer: UIGestureRecognizer,
        shouldRequireFailureOf otherGestureRecognizer: UIGestureRecognizer
    ) -> Bool {
        // Since the text field is right-aligned, a tap in the middle of it puts the caret at the front rather than at
        // the end.
        // In order to circumvent that, the tap recognizer used by the text field should be forced to fail once
        // before the text field becomes the first responder.
        // However long tap and other recognizers are unaffected, which makes it possible to tap and hold to grab
        // the cursor.
        otherGestureRecognizer.view == textField && otherGestureRecognizer.isKind(of: UITapGestureRecognizer.self)
    }

    func gestureRecognizer(
        _ gestureRecognizer: UIGestureRecognizer,
        shouldRecognizeSimultaneouslyWith otherGestureRecognizer: UIGestureRecognizer
    ) -> Bool {
        // Simultaneous recogition is a prerequisite for enabling failure requirements.
        true
    }
}

extension TextCellContentView: UITextFieldDelegate {
    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        if actualConfiguration.textFieldProperties.resignOnReturn {
            textField.resignFirstResponder()
        }
        return true
    }

    func textField(
        _ textField: UITextField,
        shouldChangeCharactersIn range: NSRange,
        replacementString string: String
    ) -> Bool {
        guard
            let currentString = textField.text,
            let stringRange = Range(range, in: currentString) else { return false }
        let updatedText = currentString.replacingCharacters(in: stringRange, with: string)

        if let maxLength = actualConfiguration.maxLength, maxLength < updatedText.count {
            return false
        }

        switch actualConfiguration.inputFilter {
        case .allowAll:
            return true
        case .digitsOnly:
            return string.allSatisfy { $0.isASCII && $0.isNumber }
        }
    }
}

extension TextCellContentConfiguration.TextFieldProperties {
    func apply(to textField: CustomTextField) {
        textField.font = font
        textField.backgroundColor = .clear
        textField.textColor = textColor
        textField.placeholderTextColor = placeholderColor
        textField.textAlignment = .right
        textField.textMargins = .zero
        textField.cornerRadius = 0
        textField.textContentType = textContentType
        textField.keyboardType = keyboardType
        textField.returnKeyType = returnKey
        textField.isSecureTextEntry = isSecureTextEntry
        textField.autocorrectionType = autocorrectionType
        textField.smartInsertDeleteType = smartInsertDeleteType
        textField.smartDashesType = smartDashesType
        textField.smartQuotesType = smartQuotesType
        textField.spellCheckingType = spellCheckingType
        textField.autocapitalizationType = autocapitalizationType
    }
}

extension TextCellContentConfiguration.EditingEvents {
    func register(in textField: UITextField) {
        onChange.map { textField.addAction($0, for: .editingChanged) }
        onBegin.map { textField.addAction($0, for: .editingDidBegin) }
        onEnd.map { textField.addAction($0, for: .editingDidEnd) }
        onEndOnExit.map { textField.addAction($0, for: .editingDidEndOnExit) }
    }

    func unregister(from textField: UITextField) {
        onChange.map { textField.removeAction($0, for: .editingChanged) }
        onBegin.map { textField.removeAction($0, for: .editingDidBegin) }
        onEnd.map { textField.removeAction($0, for: .editingDidEnd) }
        onEndOnExit.map { textField.removeAction($0, for: .editingDidEndOnExit) }
    }
}
