import SwiftUI

struct MullvadSecondaryTextField: View {
    let placeholder: LocalizedStringKey
    @Binding var text: String
    var isValid = true

    @FocusState private var isFocused: Bool
    @Environment(\.isEnabled) private var isEnabled

    var body: some View {
        HStack(spacing: 4) {
            Image.mullvadIconSearch
            TextField(
                placeholder,
                text: $text,
                prompt: Text(placeholder)
                    .foregroundColor(
                        isEnabled ? .MullvadTextField.inputPlaceholder : .MullvadTextField.textDisabled
                    )
            )
            .focused($isFocused)
            if !text.isEmpty && isEnabled {
                Button {
                    withAnimation {
                        text = ""
                    }
                } label: {
                    Image.mullvadIconCross
                }
            }
        }
        .padding(8)
        .background(
            isEnabled
                ? Color.MullvadTextField.background
                : Color.MullvadTextField
                    .backgroundDisabled
        )
        .foregroundColor(isEnabled ? .MullvadTextField.textInput : .MullvadTextField.textDisabled)
        .overlay {
            if isFocused {
                RoundedRectangle(cornerRadius: 12)
                    .inset(by: 1)
                    .stroke(
                        isValid ? Color.MullvadTextField.borderFocused : Color.MullvadTextField.borderError,
                        lineWidth: 2
                    )
            } else if isEnabled,
                !isValid
            {
                RoundedRectangle(cornerRadius: 12)
                    .inset(by: 0.5)
                    .stroke(
                        Color.MullvadTextField.borderError,
                        lineWidth: 1
                    )
            }
        }
        .clipShape(
            RoundedRectangle(cornerRadius: 12)
        )
    }
}

#Preview {
    StatefulPreviewWrapper("") { text in
        VStack {
            let placeholder = "Placeholder text"
            MullvadSecondaryTextField(
                placeholder: LocalizedStringKey(placeholder),
                text: text
            )

            MullvadSecondaryTextField(
                placeholder: LocalizedStringKey(placeholder),
                text: text,
                isValid: false
            )

            MullvadSecondaryTextField(
                placeholder: LocalizedStringKey(placeholder),
                text: text
            )
            .disabled(true)
        }
        .padding()
        .background(Color.mullvadBackground)
    }
}
