//
//  ConfigurableTextField.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct ConfigurableTextField: View {

    // MARK: - Required properties
    let title: LocalizedStringKey?
    let placeholder: LocalizedStringKey
    let leadingView: AnyView?
    let trailingView: AnyView?
    let accessibilityIdentifier: AccessibilityIdentifier?

    var font: Font = .mullvadSmall
    var foregroundColor: Color = Color.MullvadTextField.textInput
    var placeholderColor: Color = Color.MullvadTextField.inputPlaceholder
    var cornerRadius: CGFloat = 4.0
    var messageFont: Font = .mullvadTiny
    var backgroundColor: Color = Color.MullvadTextField.background

    // MARK: - Bindings
    @Binding var text: String
    @Binding var message: Message?
    @Binding var borderStyle: BorderStyle

    // MARK: - Configuration
    var configuration: TextFieldNamespace.Configuration

    @FocusState private var internalFocus: Bool
    private var externalFocus: FocusState<Bool>.Binding?

    var spacing = 4.0
    var height = 44.0

    init<Leading: View, Trailing: View>(
        title: LocalizedStringKey? = nil,
        placeholder: LocalizedStringKey,
        foregroundColor: Color = .mullvadTextPrimary,
        text: Binding<String>,
        isFocused: FocusState<Bool>.Binding? = nil,
        message: Binding<Message?> = .constant(nil),
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        borderStyle: Binding<BorderStyle>,
        configuration: TextFieldNamespace.Configuration = .init(),
        @ViewBuilder leadingView: () -> Leading = { EmptyView() },
        @ViewBuilder trailingView: () -> Trailing = { EmptyView() }
    ) {
        self.title = title
        self.placeholder = placeholder
        self.foregroundColor = foregroundColor
        self.accessibilityIdentifier = accessibilityIdentifier
        self.configuration = configuration
        self.leadingView = leadingView().typeErase()
        self.trailingView = trailingView().typeErase()
        self.externalFocus = isFocused

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

    private var suggestions: [String] {
        guard let autoComplete = configuration.autoComplete else {
            return []
        }

        guard text.isEmpty == false else {
            return autoComplete.$suggestions.wrappedValue
        }

        let suggestions = autoComplete.filteredSuggestions(for: text)

        return autoComplete.suggestions.contains(text) ? [] : suggestions
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0.0) {
            if let title = title {
                Text(title)
                    .foregroundStyle(foregroundColor)
                    .font(.mullvadTinySemiBold)
                    .padding(.bottom, spacing)
            }

            HStack {
                leadingView
                    .padding(.leading, 8.0)

                TextField(
                    "",
                    text: $text,
                    prompt: Text(placeholder)
                        .foregroundStyle(placeholderColor)
                )
                .frame(maxWidth: .infinity)
                .keyboardType(configuration.keyboardType)
                .frame(height: height)
                .focused(externalFocus ?? $internalFocus)
                .foregroundStyle(foregroundColor)
                .font(font)
                .submitLabel(configuration.submitConfiguration?.label ?? .return)
                .padding(.leading, 8.0)
                .autocorrectionDisabled()
                .textInputAutocapitalization(.never)
                .onSubmit {
                    configuration.submitConfiguration?.action()
                }
                .onChange(
                    of: text,
                    { _, newValue in
                        if let formatter = configuration.formatter {
                            let formatted = formatter.format(newValue)
                            if formatted != newValue {
                                text = formatted
                            }
                        }
                    }
                )
                .accessibilityIdentifier(accessibilityIdentifier)

                if !text.isEmpty {
                    Button {
                        text = ""
                    } label: {
                        ResizableImageView(image: .mullvadIconCross, dimension: .width(24.0))
                    }
                    .buttonStyle(.plain)
                    .padding(.trailing, 8)
                    .accessibilityLabel(Text("Clear text"))
                }

                trailingView
                    .padding(.leading, 8.0)
            }
            .background(backgroundColor)
            .modifier(
                RoundedCornerModifier(
                    cornerRadius: cornerRadius,
                    corners: suggestions.isEmpty ? .allCorners : [.topLeft, .topRight],
                    insertBy: .zero,
                    borderColor: effectiveBorderStyle.color,
                    borderWidth: effectiveBorderStyle.lineWidth
                )
            )

            if let autoComplete = configuration.autoComplete, !suggestions.isEmpty {
                SuggestionsDropdownView(
                    suggestions: suggestions,
                    onSelect: { suggestion in
                        autoComplete.onSelect(suggestion)
                        text = suggestion
                    },
                    onRemove: { suggestion in
                        autoComplete.onRemove?(suggestion)
                    }
                )
                .backgroundColor(Color.MullvadTextField.backgroundSuggestion)
                .borderStyle(.normal)
                .dividerColor(Color.MullvadTextField.border)
            }

            if let message = message {
                MessageView(message: message, font: messageFont)
                    .padding(.top, spacing)
            }
        }
        .onAppear {
            guard let formatter = configuration.formatter else { return }
            let formatted = formatter.format(text)
            text = formatted
        }

    }
}

extension ConfigurableTextField {
    func font(_ font: Font) -> Self {
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

    func focused(_ binding: FocusState<Bool>.Binding) -> Self {
        var copy = self
        copy.externalFocus = binding
        return copy
    }

    func spacing(_ spacing: CGFloat) -> Self {
        var copy = self
        copy.spacing = spacing
        return copy
    }
}

enum TextFieldNamespace {
    enum AllowedInput {
        case numeric
        case alphanumeric(isUpperCase: Bool)
    }

    struct AutoCompleteConfiguration {
        @Binding var suggestions: [String]
        let onSelect: (String) -> Void
        let onRemove: ((String) -> Void)?

        init(
            suggestions: Binding<[String]>,
            onSelect: @escaping (String) -> Void,
            onRemove: ((String) -> Void)? = nil
        ) {
            _suggestions = suggestions
            self.onSelect = onSelect
            self.onRemove = onRemove
        }

        func filteredSuggestions(for text: String) -> [String] {
            guard !text.isEmpty else { return [] }

            return _suggestions.wrappedValue.filter {
                $0.localizedCaseInsensitiveContains(text)
            }
        }
    }

    enum InputFormatter {
        case none
        case grouped(GroupedTextFormatter.FormatterConfiguration)
    }

    struct Configuration {
        var formatter: (any TextFormatting)?
        var autoComplete: AutoCompleteConfiguration?
        var keyboardType: UIKeyboardType = .default
        var submitConfiguration: SubmitConfiguration? = nil
    }

    struct SubmitConfiguration {
        let label: SubmitLabel
        let action: () -> Void
    }
}
