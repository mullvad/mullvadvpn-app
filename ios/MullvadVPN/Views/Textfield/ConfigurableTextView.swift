////
////  ConfigurableTextView.swift
////  MullvadVPN
////
////  Created by Mojgan on 2026-07-24.
////  Copyright © 2026 Mullvad VPN AB. All rights reserved.
////
///
import SwiftUI

struct ConfigurableTextView: View {
    let spacing = 4.0

    // MARK: - Required properties
    let title: String?
    let placeholder: String
    let accessibilityIdentifier: AccessibilityIdentifier?

    var font: UIFont = .mullvadSmall
    var foregroundColor: Color = Color.MullvadTextField.textInput
    var placeholderColor: Color = Color.MullvadTextField.inputPlaceholder
    var cornerRadius: CGFloat = 4.0
    var messageFont: Font = .mullvadTiny
    var backgroundColor: Color = Color.MullvadTextField.background
    var height: CGFloat = 66.0

    // MARK: - Bindings
    @Binding var text: String
    @Binding var message: Message?
    @Binding var borderStyle: BorderStyle
    @State private var internalFocus = false

    private var externalFocus: Binding<Bool>?

    init(
        title: String? = nil,
        placeholder: String,
        text: Binding<String>,
        message: Binding<Message?> = .constant(nil),
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        borderStyle: Binding<BorderStyle>
    ) {
        self.title = title
        self.placeholder = placeholder
        self.accessibilityIdentifier = accessibilityIdentifier
        self._text = text
        self._message = message
        self._borderStyle = borderStyle
    }

    private var focusBinding: Binding<Bool> {
        externalFocus ?? $internalFocus
    }

    private var isFocused: Bool {
        focusBinding.wrappedValue
    }

    private var effectiveBorderStyle: BorderStyle {
        switch borderStyle {
        case .normal:
            return isFocused ? .focused : .normal

        case .error,
            .none,
            .focused:
            return borderStyle
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0.0) {
            if let title = title {
                Text(title)
                    .foregroundStyle(foregroundColor)
                    .font(.mullvadTinySemiBold)
                    .padding(.bottom, spacing)
            }

            GrowingTextEditor(
                placeholder: placeholder,
                text: $text,
                isFocused: focusBinding,
                maxHeight: height,
                font: font,
                foregroundColor: foregroundColor,
                placeholderColor: placeholderColor,
                accessibilityIdentifier: accessibilityIdentifier
            )
            .modifier(
                RoundedCornerModifier(
                    cornerRadius: cornerRadius,
                    corners: .allCorners,
                    insertBy: .zero,
                    borderColor: effectiveBorderStyle.color,
                    borderWidth: effectiveBorderStyle.lineWidth
                )
            )
            .background(backgroundColor)

            if let message = message {
                MessageView(message: message, font: messageFont)
                    .padding(.top, spacing)
            }
        }
    }
}

extension ConfigurableTextView {
    func font(_ font: UIFont) -> Self {
        var copy = self
        copy.font = font
        return copy
    }

    func foregroundColor(_ color: Color) -> Self {
        var copy = self
        copy.foregroundColor = color
        return copy
    }

    func placeholderColor(_ color: Color) -> Self {
        var copy = self
        copy.placeholderColor = color
        return copy
    }

    func cornerRadius(_ radius: CGFloat) -> Self {
        var copy = self
        copy.cornerRadius = radius
        return copy
    }

    func messageFont(_ font: Font) -> Self {
        var copy = self
        copy.messageFont = font
        return copy
    }

    func backgroundColor(_ color: Color) -> Self {
        var copy = self
        copy.backgroundColor = color
        return copy
    }

    func height(_ height: CGFloat) -> Self {
        var copy = self
        copy.height = height
        return copy
    }

    func focused(_ binding: Binding<Bool>) -> Self {
        var copy = self
        copy.externalFocus = binding
        return copy
    }
}
