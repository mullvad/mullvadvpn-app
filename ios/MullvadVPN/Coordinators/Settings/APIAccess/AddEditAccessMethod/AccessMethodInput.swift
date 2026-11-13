// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

enum AccessMethodField: Hashable {
    case name, server, port, username, password
}

struct AccessMethodTextFieldAppearance {
    var font: Font = .mullvadSmall
    var foregroundColor: Color = .MullvadTextField.textInput
    var placeholderColor: Color = .MullvadTextField.inputPlaceholder
    var height: CGFloat = 44.0
}

struct AccessMethodTextField: View {
    let placeholder: LocalizedStringKey
    @Binding var text: String

    let field: AccessMethodField
    var focusedField: FocusState<AccessMethodField?>.Binding?
    var appearance: AccessMethodTextFieldAppearance
    var configuration: TextFieldNamespace.Configuration
    var accessibilityIdentifier: AccessibilityIdentifier?
    @State private var updateTextTask: Task<Void, Never>?

    init(
        placeholder: LocalizedStringKey,
        text: Binding<String>,
        field: AccessMethodField,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        focusedField: FocusState<AccessMethodField?>.Binding? = nil,
        appearance: AccessMethodTextFieldAppearance = .init(),
        configuration: TextFieldNamespace.Configuration = .init()
    ) {
        self.placeholder = placeholder
        self.field = field
        self.focusedField = focusedField
        self.appearance = appearance
        self.configuration = configuration

        _text = text
    }

    var body: some View {
        inputView
    }

    private var inputView: some View {
        TextField(
            "",
            text: $text,
            prompt: Text(placeholder)
                .foregroundStyle(appearance.placeholderColor)
        )
        .frame(maxWidth: .infinity)
        .keyboardType(configuration.keyboardType)
        .frame(height: appearance.height)
        .ifLet(
            focusedField,
            { view, value in
                view.focused(value, equals: field)
            }
        )
        .foregroundStyle(appearance.foregroundColor)
        .font(appearance.font)
        .submitLabel(configuration.submitConfiguration?.label ?? .return)
        .autocorrectionDisabled()
        .textInputAutocapitalization(.never)
        .padding(.horizontal, 8.0)
        .onSubmit {
            configuration.submitConfiguration?.action()
        }
        .onChange(of: text, { onChangeOfText($1) })
        .accessibilityIdentifier(accessibilityIdentifier)
    }

    func onChangeOfText(_ newValue: String) {
        // Debounce text updates for the users that type too fast on slow phones
        updateTextTask?.cancel()
        updateTextTask = Task {
            guard Task.isCancelled == false else { return }
            formatText(newValue)
        }
    }

    func formatText(_ newValue: String) {
        if let formatter = configuration.formatter {
            let formatted = formatter.format(newValue)
            if formatted != newValue {
                text = formatted
            }
        }
    }
}
