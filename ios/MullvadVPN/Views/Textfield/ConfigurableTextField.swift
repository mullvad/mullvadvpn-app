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

    // MARK: - Bindings
    @Binding var text: String
    @Binding var message: MessageView.Message?
    @Binding var borderStyle: BorderStyle

    // MARK: - Appearance
    var appearance: InputViewAppearance

    // MARK: - Configuration
    var configuration: TextFieldNamespace.Configuration

    @Environment(\.isEnabled) private var isEnabled
    @FocusState private var internalFocus: Bool
    private var externalFocus: FocusState<Bool>.Binding?
    @State private var animatedMessage: MessageView.Message?
    @State private var animatedSuggestions: [String] = []

    init<Leading: View, Trailing: View>(
        title: LocalizedStringKey? = nil,
        placeholder: LocalizedStringKey,
        text: Binding<String>,
        isFocused: FocusState<Bool>.Binding? = nil,
        message: Binding<MessageView.Message?> = .constant(nil),
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        borderStyle: Binding<BorderStyle>,
        appearance: InputViewAppearance = .init(height: 44.0),
        configuration: TextFieldNamespace.Configuration = .init(),
        @ViewBuilder leadingView: () -> Leading = { EmptyView() },
        @ViewBuilder trailingView: () -> Trailing = { EmptyView() }
    ) {
        self.title = title
        self.placeholder = placeholder
        self.appearance = appearance
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
            titleView
            HStack {
                leadingView
                inputView
                clearButton
                trailingView
            }
            .background(appearance.backgroundColor)
            .modifier(
                RoundedCornerModifier(
                    cornerRadius: appearance.cornerRadius,
                    corners: animatedSuggestions.isEmpty ? .allCorners : [.topLeft, .topRight],
                    insertBy: .zero,
                    borderColor: effectiveBorderStyle.color,
                    borderWidth: effectiveBorderStyle.lineWidth
                )
            )

            dropdownView
            messageView
        }
        .onChange(of: suggestions) { _, newSuggestions in
            withAnimation(.easeInOut) {
                animatedSuggestions = newSuggestions
            }
        }
        .onChange(of: message != nil) { _, hasMessage in
            withAnimation(.easeInOut) {
                animatedMessage = hasMessage ? message : nil
            }
        }
        .onAppear {
            animatedSuggestions = suggestions
            animatedMessage = message
            guard let formatter = configuration.formatter else { return }
            let formatted = formatter.format(text)
            text = formatted
        }
    }

    @ViewBuilder private var titleView: some View {
        if let title = title {
            Text(title)
                .foregroundStyle(appearance.foregroundColor)
                .font(appearance.titleFont)
                .padding(.bottom, appearance.spacing)
        }
    }

    @ViewBuilder private var inputView: some View {
        TextField(
            "",
            text: $text,
            prompt: Text(placeholder)
                .foregroundStyle(appearance.placeholderColor)
        )
        .frame(maxWidth: .infinity)
        .keyboardType(configuration.keyboardType)
        .frame(height: appearance.height)
        .focused(externalFocus ?? $internalFocus)
        .foregroundStyle(appearance.foregroundColor)
        .font(appearance.font)
        .submitLabel(configuration.submitConfiguration?.label ?? .return)
        .autocorrectionDisabled()
        .textInputAutocapitalization(.never)
        .padding(.horizontal, 8.0)
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
    }

    @ViewBuilder private var clearButton: some View {
        if !text.isEmpty {
            Button {
                text = ""
            } label: {
                ResizableImageView(image: .mullvadIconCross, dimension: .width(24.0))
            }
            .buttonStyle(.plain)
            .padding(.horizontal, 8.0)
            .accessibilityLabel(Text("Clear text"))
        }
    }

    @ViewBuilder private var dropdownView: some View {
        if let autoComplete = configuration.autoComplete, !animatedSuggestions.isEmpty, isEnabled {
            SuggestionsDropdownView(
                suggestions: animatedSuggestions,
                appearance: autoComplete.appearance,
                onSelect: { suggestion in
                    autoComplete.onSelect(suggestion)
                    text = suggestion
                },
                onRemove: { suggestion in
                    autoComplete.onRemove?(suggestion)
                }
            )
            .transition(.opacity)
        }
    }

    @ViewBuilder private var messageView: some View {
        if let message = animatedMessage {
            MessageView(message: message, font: appearance.messageFont)
                .padding(.top, appearance.spacing)
                .transition(.opacity)
        }
    }

}

enum TextFieldNamespace {
    enum AllowedInput {
        case numeric
        case alphanumeric(isUpperCase: Bool)
    }

    struct AutoCompleteConfiguration {
        @Binding var suggestions: [String]
        let appearance: SuggestionsDropdownViewAppearance
        let onSelect: (String) -> Void
        let onRemove: ((String) -> Void)?

        init(
            suggestions: Binding<[String]>,
            appearance: SuggestionsDropdownViewAppearance = .init(),
            onSelect: @escaping (String) -> Void,
            onRemove: ((String) -> Void)? = nil
        ) {
            _suggestions = suggestions
            self.appearance = appearance
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
