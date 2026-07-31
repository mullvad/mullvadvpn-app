//
//  PreviewInputViews.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

#Preview("Input views") {

    @Previewable
    @State
    var suggestions = [
        "1234567890012121",
        "1234567790012121",
    ].map(GroupedTextFormatter.accountNumber.format)

    VStack(spacing: 24) {
        PreviewTextField(
            placeholder: "Placeholder text",
            text: "",
            message: .init(
                text: "Supporting text",
                appearance: .info
            ),
            foregroundColor: .mullvadTextPrimary,
            borderStyle: .normal,
            configuration: .init(keyboardType: .default),
            leadingView: {
                EmptyView()
            },
            trailingView: {
                EmptyView()
            })

        PreviewTextField(
            placeholder: "Placeholder text",
            text: "Input Test",
            message: .init(
                text: "Supporting text",
                appearance: .info
            ),
            foregroundColor: .mullvadTextPrimary,
            borderStyle: .normal,
            configuration: .init(keyboardType: .default),
            leadingView: {
                EmptyView()
            },
            trailingView: { EmptyView() })

        PreviewTextField(
            placeholder: "Placeholder text",
            text: "",
            message: .init(
                text: "Error message",
                appearance: .error
            ),
            foregroundColor: .mullvadTextPrimary,
            borderStyle: .error,
            configuration: .init(keyboardType: .default),
            leadingView: {
                EmptyView()
            },
            trailingView: { EmptyView() })

        PreviewTextField(
            placeholder: "Account Number",
            text: "",
            message: .init(
                text: "Supporting text",
                appearance: .info
            ),
            foregroundColor: .mullvadTextPrimary,
            borderStyle: .normal,
            configuration: .init(
                formatter: GroupedTextFormatter.accountNumber,
                autoComplete: TextFieldNamespace.AutoCompleteConfiguration(
                    suggestions: $suggestions,
                    onSelect: { item in
                        print("\(item) is selected")
                    },
                    onRemove: { item in
                        suggestions.removeAll { $0 == item }
                    }),
                keyboardType: .numberPad),
            leadingView: {
                EmptyView()
            },
            trailingView: {
                EmptyView()
            })

        PreviewTextView(
            placeholder: "Enter your text here",
            text: "",
            message: Message(text: "Supporting text", appearance: .info),
            foregroundColor: .mullvadTextPrimary, borderStyle: .normal)
    }
    .padding()
    .background(Color.mullvadBackground)
}

private struct PreviewTextField<Leading: View, Trailing: View>: View {
    let placeholder: String
    @State private var text = "550E8400-E29B-41D4-A716-446655440000"
    @State private var message: Message?
    @State private var borderStyle: BorderStyle
    let foregroundColor: Color

    let configuration: TextFieldNamespace.Configuration

    let leadingView: Leading
    let trailingView: Trailing

    init(
        placeholder: String,
        text: String,
        message: Message?,
        foregroundColor: Color,
        borderStyle: BorderStyle,
        configuration: TextFieldNamespace.Configuration,
        @ViewBuilder leadingView: () -> Leading,
        @ViewBuilder trailingView: () -> Trailing
    ) {
        self.placeholder = placeholder
        self.foregroundColor = foregroundColor
        self.leadingView = leadingView()
        self.trailingView = trailingView()
        self.configuration = configuration
        _text = State(initialValue: text)
        _message = State(initialValue: message)
        _borderStyle = State(initialValue: borderStyle)

    }

    var body: some View {
        ConfigurableTextField(
            title: "Label",
            placeholder: LocalizedStringKey(placeholder),
            foregroundColor: foregroundColor,
            text: $text,
            message: $message,
            borderStyle: $borderStyle,
            configuration: configuration,
            leadingView: { leadingView },
            trailingView: { trailingView }
        )
    }
}

private struct PreviewTextView: View {
    let placeholder: String
    @State private var text = ""
    @State private var message: Message?
    @State private var borderStyle: BorderStyle
    @State private var isFocused: Bool
    let foregroundColor: Color

    init(
        placeholder: String,
        text: String,
        message: Message?,
        foregroundColor: Color,
        borderStyle: BorderStyle
    ) {
        self.placeholder = placeholder
        self.foregroundColor = foregroundColor
        _text = State(initialValue: text)
        _message = State(initialValue: message)
        _borderStyle = State(initialValue: borderStyle)
        _isFocused = State(initialValue: false)
    }

    var body: some View {
        ConfigurableTextView(
            title: "TextView",
            placeholder: placeholder,
            text: $text,
            message: $message,
            borderStyle: $borderStyle
        )
        .focused($isFocused)
    }
}
