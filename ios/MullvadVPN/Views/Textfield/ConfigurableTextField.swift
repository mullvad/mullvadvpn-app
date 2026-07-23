//
//  ConfigurableTextField.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct ConfigurableTextField<Leading: View, Trailing: View>: View {

    // MARK: - Required properties
    let title: String
    let placeHolder: String
    let foregroundColor: Color
    let leadingView: Leading
    let trailingView: Trailing
    let accessibilityIdentifier: AccessibilityIdentifier?

    // MARK: - Bindings
    @Binding var text: String
    @Binding var message: ConfigurableTextFieldNamespace.Message?
    @Binding var borderStyle: ConfigurableTextFieldNamespace.BorderStyle

    // MARK: - Configuration
    var configuration: ConfigurableTextFieldNamespace.TextFieldConfiguration

    @FocusState private var isFocused: Bool
    @State private var suggestions: [String] = []

    init(
        title: String = "",
        placeHolder: String,
        foregroundColor: Color = .mullvadTextPrimary,
        text: Binding<String>,
        message: Binding<ConfigurableTextFieldNamespace.Message?> = .constant(nil),
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        borderStyle: Binding<ConfigurableTextFieldNamespace.BorderStyle>,
        configuration: ConfigurableTextFieldNamespace.TextFieldConfiguration = .init(),
        @ViewBuilder leadingView: () -> Leading,
        @ViewBuilder trailingView: () -> Trailing
    ) {
        self.title = title
        self.placeHolder = placeHolder
        self.foregroundColor = foregroundColor
        self.accessibilityIdentifier = accessibilityIdentifier
        self.configuration = configuration
        self.leadingView = leadingView()
        self.trailingView = trailingView()

        self._text = text
        self._message = message
        self._borderStyle = borderStyle
    }

    var filteredSuggestions: [String] {
        configuration.autoComplete?
            .filteredSuggestions(for: text) ?? []
    }

    private var hasTrailingView: Bool {
        Trailing.self != EmptyView.self
    }

    private var effectiveBorderStyle: ConfigurableTextFieldNamespace.BorderStyle {
        switch borderStyle {
        case .normal:
            return isFocused ? .focused : .normal

        case .error,
            .success,
            .disabled,
            .focused:
            return borderStyle
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0.0) {
            Text(title)
                .foregroundStyle(TextFieldMetrics.primaryTextcolor)
                .font(.body)

            HStack {
                leadingView

                TextField(
                    "",
                    text: $text,
                    prompt: Text(placeHolder)
                        .foregroundStyle(TextFieldMetrics.placeholderTextcolor)
                )
                .frame(maxWidth: .infinity)
                .keyboardType(configuration.keyboardType)
                .frame(minHeight: TextFieldMetrics.minHeight)
                .focused($isFocused)
                .foregroundStyle(foregroundColor)
                .font(TextFieldMetrics.fontSize)
                .submitLabel(configuration.submitConfiguration?.label ?? .return)
                .onSubmit {
                    configuration.submitConfiguration?.action()
                }
                .onChange(
                    of: text,
                    { _, newValue in
                        var value = newValue

                        if let formatter = configuration.formatter {
                            let formatted = formatter.format(newValue)
                            if formatted != newValue {
                                text = formatted
                                value = formatted
                            }
                        }

                        if let autoComplete = configuration.autoComplete {
                            suggestions = autoComplete.filteredSuggestions(for: value)
                        }
                    }
                )
                .accessibilityIdentifier(accessibilityIdentifier)

                if hasTrailingView {
                    trailingView
                } else if !text.isEmpty {
                    Button {
                        text = ""
                    } label: {
                        ResizableImageView(image: .mullvadIconCross, dimension: .width(24.0))
                    }
                    .buttonStyle(.plain)
                }
            }
            .modifier(
                TextFieldBorderModifier(
                    style: effectiveBorderStyle,
                    cornerRadius: TextFieldMetrics.cornerRadius)
            )
            .padding(.horizontal, effectiveBorderStyle.lineWidth)
            .padding(.top, TextFieldMetrics.spacing)

            if !suggestions.isEmpty {
                AutoCompleteSuggestionsView(suggestions: suggestions) { suggestion in
                    text = suggestion
                    suggestions.removeAll()
                }
            }

            if let message = message {
                FieldMessageView(message: message)
                    .padding(.top, TextFieldMetrics.spacing)
            }
        }
        .onAppear {
            guard let formatter = configuration.formatter else { return }
            let formatted = formatter.format(text)
            text = formatted
        }
    }
}

enum ConfigurableTextFieldNamespace {
    enum AllowedInput {
        case numeric
        case alphanumeric(isUpperCase: Bool)
    }

    enum BorderStyle {
        case normal
        case focused
        case error
        case success
        case disabled

        var color: Color {
            switch self {
            case .normal:
                Color(red: 248.0 / 255.0, green: 241.0 / 255.0, blue: 241.0 / 255.0)
            case .focused:
                Color(red: 248.0 / 255.0, green: 247.0 / 255.0, blue: 241.0 / 255.0)
            case .error:
                Color(red: 235.0 / 255.0, green: 93.0 / 255.0, blue: 64.0 / 255.0)
            case .success:
                Color(red: 248.0 / 255.0, green: 241.0 / 255.0, blue: 241.0 / 255.0)
            case .disabled:
                Color(red: 71.0 / 255.0, green: 88.0 / 255.0, blue: 108.0 / 255.0)
            }
        }

        var lineWidth: CGFloat {
            switch self {
            case .focused, .error:
                2.0
            default:
                1.0
            }
        }
    }

    struct AutoCompleteConfiguration {
        let suggestions: [String]

        func filteredSuggestions(for text: String) -> [String] {
            guard !text.isEmpty else { return [] }

            return suggestions.filter {
                $0.localizedCaseInsensitiveContains(text)
            }
        }
    }

    struct Message: Equatable {
        let text: String
        let appearance: Appearance

        struct Appearance: Equatable {
            let foregroundColor: Color
            let icon: Image?

            static let info: Self = .init(
                foregroundColor: TextFieldMetrics.primaryTextcolor,
                icon: .mullvadIconInfo
            )

            static let success: Self = .init(
                foregroundColor: TextFieldMetrics.primaryTextcolor,
                icon: .mullvadIconSuccess
            )

            static let error: Self = .init(
                foregroundColor: TextFieldMetrics.primaryTextcolor,
                icon: .mullvadIconError
            )
        }
    }

    enum InputFormatter {
        case none
        case grouped(GroupedTextFormatter.FormatterConfiguration)
    }

    struct SubmitConfiguration {
        let label: SubmitLabel
        let action: () -> Void
    }

    struct TextFieldConfiguration {
        var formatter: (any TextFormatting)?
        var autoComplete: AutoCompleteConfiguration?
        var keyboardType: UIKeyboardType = .default
        var submitConfiguration: SubmitConfiguration? = nil
    }
}
extension ConfigurableTextField {
    private struct FieldMessageView: View {
        let message: ConfigurableTextFieldNamespace.Message

        var body: some View {
            HStack(spacing: 4.0) {
                if let icon = message.appearance.icon {
                    ResizableImageView(image: icon, dimension: .width(18))
                }
                Text(message.text)
                    .font(TextFieldMetrics.messageFont)
                    .foregroundStyle(message.appearance.foregroundColor)

                Spacer()
            }
        }

    }

    private struct TextFieldBorderModifier: ViewModifier {
        let style: ConfigurableTextFieldNamespace.BorderStyle
        let cornerRadius: CGFloat

        func body(content: Content) -> some View {
            content
                .padding(.horizontal, 8)
                .overlay {
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .stroke(style.color, lineWidth: style.lineWidth)
                }
        }
    }

    struct AutoCompleteSuggestionsView: View {
        let suggestions: [String]
        let onSelect: (String) -> Void

        private var suggestionsHeight: CGFloat {
            let dividerHeight = max(0, suggestions.count - 1)
            let totalHeight =
                CGFloat(suggestions.count) * TextFieldMetrics.minHeight + CGFloat(dividerHeight)

            return min(totalHeight, 200)
        }

        var body: some View {
            ScrollView {
                LazyVStack(spacing: 0) {
                    ForEach(Array(suggestions.enumerated()), id: \.element) { index, suggestion in
                        Button {
                            onSelect(suggestion)
                        } label: {
                            Text(suggestion)
                                .frame(
                                    maxWidth: .infinity,
                                    minHeight: TextFieldMetrics.minHeight,
                                    alignment: .leading
                                )
                                .font(TextFieldMetrics.fontSize)
                                .foregroundStyle(TextFieldMetrics.primaryTextcolor)
                                .padding(.horizontal, 8.0)
                        }
                        .buttonStyle(.plain)

                        if index < suggestions.count - 1 {
                            Divider()
                                .background(TextFieldMetrics.borderColor)

                        }
                    }
                }
            }
            .frame(height: suggestionsHeight)
            .background {
                UnevenRoundedRectangle(
                    bottomLeadingRadius: TextFieldMetrics.cornerRadius,
                    bottomTrailingRadius: TextFieldMetrics.cornerRadius
                )
                .fill(TextFieldMetrics.backgroundColor)
            }
            .overlay {
                UnevenRoundedRectangle(
                    bottomLeadingRadius: TextFieldMetrics.cornerRadius,
                    bottomTrailingRadius: TextFieldMetrics.cornerRadius
                )
                .stroke(TextFieldMetrics.borderColor)
            }
            .clipShape(
                UnevenRoundedRectangle(
                    bottomLeadingRadius: TextFieldMetrics.cornerRadius,
                    bottomTrailingRadius: TextFieldMetrics.cornerRadius
                )
            )
        }
    }
}

private enum TextFieldMetrics {
    static let minHeight: CGFloat = 44.0
    static let cornerRadius: CGFloat = 4.0
    static let spacing: CGFloat = 4.0
    static let primaryTextcolor: Color = .mullvadTextPrimary
    static let placeholderTextcolor: Color = .secondaryTextColor
    static let fontSize: Font = .mullvadSmall
    static let messageFont: Font = .mullvadTiny
    static let backgroundColor: Color = .mullvadBackground
    static let borderColor: Color = ConfigurableTextFieldNamespace.BorderStyle.normal.color
}
