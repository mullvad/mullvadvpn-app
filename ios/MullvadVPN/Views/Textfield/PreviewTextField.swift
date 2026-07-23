//
//  PreviewTextField.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

#Preview {
    VStack(spacing: 24) {
        let groupedTextFormatter = GroupedTextFormatter(
            configuration: .init(
                allowedInput: .alphanumeric(isUpperCase: true),
                groupSeparator: " ",
                groupSize: 4,
                maxGroups: 4
            )
        )

        PreviewTextField(
            placeHolder: "Placeholder text",
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
            placeHolder: "Placeholder text",
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
            placeHolder: "Placeholder text",
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
            placeHolder: "Account Number",
            text: "123456",
            message: .init(
                text: "Supporting text",
                appearance: .info
            ),
            foregroundColor: .mullvadTextPrimary,
            borderStyle: .normal,
            configuration: .init(
                formatter: groupedTextFormatter,
                autoComplete: ConfigurableTextFieldNamespace.AutoCompleteConfiguration(suggestions: [
                    groupedTextFormatter.format("1234567890012121"),
                    groupedTextFormatter.format("1234567890012123"),
                ]),
                keyboardType: .numberPad),
            leadingView: {
                EmptyView()
            },
            trailingView: {
                EmptyView()
            })

        PreviewTextField(
            placeHolder: "Placeholder text",
            text: "1234444444444444",
            message: .init(
                text: "Supporting text",
                appearance: .info
            ),
            foregroundColor: .mullvadTextPrimary,
            borderStyle: .normal,
            configuration: .init(
                formatter: groupedTextFormatter,
                keyboardType: .numberPad),
            leadingView: {
                EmptyView()
            },
            trailingView: {
                EmptyView()
            })
    }
    .padding()
    .background(Color.mullvadBackground)
}

private struct PreviewTextField<Leading: View, Trailing: View>: View {
    let placeHolder: String
    @State private var text = "550E8400-E29B-41D4-A716-446655440000"
    @State private var message: ConfigurableTextFieldNamespace.Message?
    @State private var borderStyle: ConfigurableTextFieldNamespace.BorderStyle
    let foregroundColor: Color

    let configuration: ConfigurableTextFieldNamespace.TextFieldConfiguration

    let leadingView: Leading
    let trailingView: Trailing

    init(
        placeHolder: String,
        text: String,
        message: ConfigurableTextFieldNamespace.Message?,
        foregroundColor: Color,
        borderStyle: ConfigurableTextFieldNamespace.BorderStyle,
        configuration: ConfigurableTextFieldNamespace.TextFieldConfiguration,
        @ViewBuilder leadingView: () -> Leading,
        @ViewBuilder trailingView: () -> Trailing
    ) {
        self.placeHolder = placeHolder
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
            placeHolder: placeHolder,
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
