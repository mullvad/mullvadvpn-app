import SwiftUI

struct MullvadPrimaryTextField: View {
    private let label: String
    private let placeholder: String
    @Binding private var text: String
    @Binding private var suggestion: String?
    private let validate: ((String) -> Bool)?

    init(
        label: String,
        placeholder: String,
        text: Binding<String>,
        suggestion: Binding<String?>? = nil,
        validate: ((String) -> Bool)? = nil
    ) {
        self.label = label
        self.placeholder = placeholder
        self._text = text
        self._suggestion = suggestion ?? .constant(nil)
        self.validate = validate
    }

    var isValid: Bool {
        validate?(text) ?? true
    }

    @FocusState private var isFocused: Bool
    @Environment(\.isEnabled) private var isEnabled

    private var showSuggestion: Bool {
        if let suggestion,
           !suggestion.isEmpty,
           suggestion != text,
           isEnabled {
            return true
        }
        return false
    }

    var body: some View {
        VStack(alignment: .leading) {
            Text(label)
                .foregroundColor(.MullvadTextField.label)
            VStack(spacing: 0) {
                HStack(spacing: 4) {
                    TextField(
                        placeholder,
                        text: $text,
                        prompt: Text(placeholder)
                            .foregroundColor(
                                isEnabled ? .MullvadTextField.inputPlaceholder : .MullvadTextField.textDisabled
                            )
                    )
                    .focused($isFocused)
                    .padding(.vertical, 12)
                    if !text.isEmpty && isEnabled {
                        Button {
                            withAnimation {
                                text = ""
                            }
                        } label: {
                            Image.mullvadIconCross
                        }
                        .padding(0)
                    }
                }
                .zIndex(1)
                .padding(.horizontal, 8)
                .background(
                    isEnabled
                        ? Color.MullvadTextField.background
                        : Color.MullvadTextField
                            .backgroundDisabled
                )
                .foregroundColor(isEnabled ? .MullvadTextField.textInput : .MullvadTextField.textDisabled)
                .overlay {
                    if isFocused {
                        RoundedCorner(
                            cornerRadius: 4,
                            corners: !showSuggestion
                                ? [.allCorners]
                                : [
                                    .topLeft,
                                    .topRight,
                                ]
                        )
                        .stroke(
                            isValid
                                ? Color.MullvadTextField.borderFocused
                                : Color.MullvadTextField.borderError,
                            lineWidth: 4
                        )
                    } else if isEnabled {
                        RoundedCorner(
                            cornerRadius: 4,
                            corners: !showSuggestion
                                ? [.allCorners]
                                : [
                                    .topLeft,
                                    .topRight,
                                ]
                        )
                        .stroke(
                            isValid
                                ? Color.MullvadTextField.border
                                : Color.MullvadTextField.borderError,
                            lineWidth: 2
                        )
                    }
                }
                .clipShape(RoundedCorner(
                    cornerRadius: 4,
                    corners: !showSuggestion
                        ? [.allCorners]
                        : [
                            .topLeft,
                            .topRight,
                        ]
                ))

                if showSuggestion,
                   let suggestion {
                    HStack {
                        Button {
                            withAnimation {
                                text = suggestion
                            }
                        } label: {
                            Text(suggestion)
                                .foregroundColor(.MullvadTextField.textInput)
                            Spacer()
                        }
                        Button {
                            withAnimation {
                                self.suggestion = nil
                            }
                        } label: {
                            Image.mullvadIconCross
                        }
                    }
                    .transition(.move(edge: .top))
                    .padding(.horizontal, 8)
                    .padding(.vertical, 12)
                    .background(Color.MullvadTextField.backgroundSuggestion)
                }
            }
            .clipShape(
                RoundedCorner(cornerRadius: 4)
            )
        }
        .transformEffect(.identity)
        .animation(.default, value: showSuggestion)
    }
}

private struct RoundedCorner: Shape {
    var cornerRadius: CGFloat = .infinity
    var corners: UIRectCorner = .allCorners
    var insertBy: CGFloat = 0

    func path(in rect: CGRect) -> Path {
        let insetRect = rect.insetBy(dx: insertBy, dy: insertBy)
        let path = UIBezierPath(
            roundedRect: insetRect,
            byRoundingCorners: corners,
            cornerRadii: CGSize(width: cornerRadius, height: cornerRadius)
        )
        return Path(path.cgPath)
    }
}

@available(iOS 17.0, *)
#Preview {
    @Previewable @State var suggestion: String? = "1234"
    @Previewable @State var text = ""
    VStack {
        MullvadPrimaryTextField(
            label: "Label",
            placeholder: "Placeholder text",
            text: $text,
            suggestion: $suggestion
        )

        MullvadPrimaryTextField(
            label: "Label",
            placeholder: "Placeholder text",
            text: $text,
            suggestion: $suggestion,
            validate: { _ in
                false
            }
        )

        MullvadPrimaryTextField(
            label: "Label",
            placeholder: "Placeholder text",
            text: $text,
            suggestion: $suggestion
        )
        .disabled(true)
    }
    .padding()
    .background(Color.yellow)
}

class UIMullvadPrimaryTextField: UIHostingController<UIMullvadPrimaryTextField.Wrapper> {
    var text: String {
        get {
            rootView.text
        }
        set {
            rootView.text = newValue
        }
    }

    struct Wrapper: View {
        let label: String
        let placeholder: String
        @State var text = ""
        @State var suggestion: String?
        let validate: ((String) -> Bool)?
        var contentType: UITextContentType?
        var keyboardType: UIKeyboardType = .default
        var submitLabel: SubmitLabel?
        var body: some View {
            MullvadPrimaryTextField(
                label: label,
                placeholder: placeholder,
                text: $text,
                suggestion: $suggestion,
                validate: validate
            )
            .textContentType(contentType)
            .keyboardType(keyboardType)
            .apply {
                if let submitLabel {
                    $0.submitLabel(submitLabel)
                } else {
                    $0
                }
            }
        }
    }

    init(
        label: String,
        placeholder: String,
        validate: ((String) -> Bool)? = nil,
        contentType: UITextContentType? = nil,
        keyboardType: UIKeyboardType = .default
    ) {
        let rootView = Wrapper(
            label: label,
            placeholder: placeholder,
            validate: validate,
            contentType: contentType,
            keyboardType: keyboardType
        )

        super.init(rootView: rootView)
    }

    override func viewDidLoad() {
        view.backgroundColor = .clear
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("Not implemented")
    }
}

struct UIMullvadPrimaryTextFieldRepresentable: UIViewRepresentable {
    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeUIView(context: Context) -> UIView {
        let controller = UIMullvadPrimaryTextField(label: "Label", placeholder: "Placeholder")
        context.coordinator.controller = controller
        return controller.view
    }

    func updateUIView(_ uiView: UIView, context: Context) {}

    class Coordinator {
        var controller: UIMullvadPrimaryTextField?
    }
}

#Preview {
    UIMullvadPrimaryTextFieldRepresentable()
        .padding()
        .background(Color.yellow)
}
