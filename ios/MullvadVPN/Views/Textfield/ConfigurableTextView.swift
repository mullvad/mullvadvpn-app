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
    // MARK: - Required properties
    let title: LocalizedStringKey?
    let placeholder: LocalizedStringKey
    let accessibilityIdentifier: AccessibilityIdentifier?

    // MARK: - Appearance
    var appearance: InputViewAppearance

    // MARK: - Bindings
    @Binding var text: String
    @Binding var message: MessageView.Message?
    @Binding var borderStyle: BorderStyle

    @FocusState private var internalFocus: Bool
    private var externalFocus: FocusState<Bool>.Binding?

    init(
        title: LocalizedStringKey? = nil,
        placeholder: LocalizedStringKey,
        text: Binding<String>,
        isFocused: FocusState<Bool>.Binding? = nil,
        message: Binding<MessageView.Message?> = .constant(nil),
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        appearance: InputViewAppearance = .init(height: 74.0),
        borderStyle: Binding<BorderStyle>
    ) {
        self.title = title
        self.placeholder = placeholder
        self.accessibilityIdentifier = accessibilityIdentifier
        self.externalFocus = isFocused
        self.appearance = appearance
        self._text = text
        self._message = message
        self._borderStyle = borderStyle
    }

    private var isFocused: Bool {
        externalFocus?.wrappedValue ?? internalFocus
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
                    .foregroundStyle(appearance.foregroundColor)
                    .font(appearance.titleFont)
                    .padding(.bottom, appearance.spacing)
            }

            ZStack(alignment: .topLeading) {
                TextEditor(text: $text)
                    .focused(externalFocus ?? $internalFocus)
                    .accessibilityIdentifier(accessibilityIdentifier)
                    .foregroundStyle(appearance.foregroundColor)
                    .font(appearance.font)
                    .scrollContentBackground(.hidden)
                    .padding(4.0)

                Text(placeholder)
                    .font(appearance.font)
                    .foregroundStyle(appearance.placeholderColor)
                    .padding(.horizontal, 8.0)
                    .padding(.vertical, 12.0)
                    .opacity(text.isEmpty ? 1.0 : 0.0)
            }
            .background(appearance.backgroundColor)
            .modifier(
                RoundedCornerModifier(
                    cornerRadius: appearance.cornerRadius,
                    corners: .allCorners,
                    insertBy: .zero,
                    borderColor: effectiveBorderStyle.color,
                    borderWidth: effectiveBorderStyle.lineWidth
                )
            )

            if let message = message {
                MessageView(message: message, font: appearance.messageFont)
                    .padding(.top, appearance.spacing)
            }
        }
    }
}
