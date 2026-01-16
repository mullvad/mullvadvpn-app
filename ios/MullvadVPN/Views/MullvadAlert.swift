import SwiftUI

struct MullvadAlert: Identifiable {
    enum AlertType {
        case info
        case warning
        case error
    }

    enum ActionType {
        case danger
        case normal
    }

    enum DismissButtonPosition {
        case top
        case bottom
    }

    struct Action: Identifiable {
        let id = UUID()
        let type: MainButtonStyle.Style
        let title: LocalizedStringKey
        let identifier: AccessibilityIdentifier?
        let handler: () async -> Void

        init(
            type: MainButtonStyle.Style,
            title: LocalizedStringKey,
            identifier: AccessibilityIdentifier? = nil,
            handler: @escaping () async -> Void
        ) {
            self.type = type
            self.title = title
            self.identifier = identifier
            self.handler = handler
        }
    }

    let id = UUID()
    let type: AlertType
    let messages: [LocalizedStringKey]
    let actions: [Action]

    init(
        type: AlertType,
        messages: [LocalizedStringKey],
        actions: [Action] = []
    ) {
        self.type = type
        self.messages = messages
        self.actions = actions
    }
}

struct MullvadInputAlert: Identifiable {
    struct Action {
        let type: MainButtonStyle.Style
        let title: LocalizedStringKey
        let identifier: AccessibilityIdentifier?
        let handler: (String) async -> Void
    }

    let id = UUID()
    let title: LocalizedStringKey
    let placeholder: LocalizedStringKey
    let action: Action
    let validate: ((String) -> Bool)?
    let dismissButtonTitle: LocalizedStringKey
}

struct AlertModifier: ViewModifier {
    @Binding var alert: MullvadAlert?
    @State var loading = false
    func body(content: Content) -> some View {
        content
            .fullScreenCover(item: $alert) { alert in
                alertView(for: alert)
            }
            .transaction {
                $0.disablesAnimations = true
            }
    }

    @ViewBuilder
    private func alertView(for alert: MullvadAlert) -> some View {
        VStack {
            Spacer()
            alertContent(for: alert)
            Spacer()
        }
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier(.alertContainerView)
        .padding()
        .background(ClearBackgroundView())
    }

    @ViewBuilder
    private func alertContent(for alert: MullvadAlert) -> some View {
        VStack(spacing: 16) {
            alertIcon(for: alert.type)
            alertMessage(alert.messages)
            VStack(spacing: 16) {
                ForEach(alert.actions) { action in
                    alertAction(for: action)
                }
            }
        }
        .padding()
        .background(Color.mullvadBackground)
        .cornerRadius(8)
    }

    @ViewBuilder
    private func alertIcon(for type: MullvadAlert.AlertType) -> some View {
        switch type {
        case .info:
            Image.mullvadIconInfo
                .resizable()
                .frame(width: 48, height: 48)
        case .error, .warning:
            Image.mullvadIconAlert
                .resizable()
                .frame(width: 48, height: 48)
        }
    }

    @ViewBuilder
    private func alertMessage(_ messages: [LocalizedStringKey]) -> some View {
        VStack {
            ForEach(Array(messages.enumerated()), id: \.offset) { _, text in
                HStack {
                    Text(text)
                        .font(.mullvadSmall)
                        .foregroundColor(.mullvadTextPrimary.opacity(0.6))
                    Spacer()
                }
            }
        }
    }

    @ViewBuilder
    private func alertAction(for action: MullvadAlert.Action) -> some View {
        MainButton(
            text: action.title,
            style: action.type,
            action: {
                Task {
                    loading = true
                    await action.handler()
                    loading = false
                }
            }
        )
        .accessibilityIdentifier(action.identifier)
    }
}

struct InputAlertModifier: ViewModifier {
    @Binding var alert: MullvadInputAlert?
    @State var loading = false
    @State var text = ""

    func body(content: Content) -> some View {
        content
            .fullScreenCover(item: $alert) { alert in
                VStack {
                    Spacer()
                    VStack(alignment: .leading, spacing: 16) {
                        Text(alert.title)
                            .font(.mullvadLarge)
                            .foregroundStyle(Color.mullvadTextPrimary)
                            .lineLimit(nil)
                            .fixedSize(horizontal: false, vertical: true)
                        MullvadPrimaryTextField(
                            label: "",
                            placeholder: alert.placeholder,
                            text: $text,
                            isFocused: .constant(true),
                            validate: alert.validate
                        )
                        VStack(spacing: 16) {
                            MainButton(
                                text: alert.action.title,
                                style: alert.action.type,
                                action: {
                                    Task {
                                        loading = true
                                        await alert.action.handler(text)
                                        loading = false
                                    }
                                }
                            )
                            .disabled(!(alert.validate?(text) ?? true))
                            .accessibilityIdentifier(alert.action.identifier)
                            MainButton(
                                text: alert.dismissButtonTitle,
                                style: .default,
                                action: { self.alert = nil }
                            )
                        }
                    }
                    .padding()
                    .background(Color.mullvadBackground)
                    .cornerRadius(8)
                    Spacer()
                }
                .onAppear {
                    text = ""
                }
                .accessibilityElement(children: .contain)
                .accessibilityIdentifier(.alertContainerView)
                .padding()
                .background(ClearBackgroundView())
            }
            .transaction {
                $0.disablesAnimations = true
            }
    }
}

extension View {
    func mullvadAlert(item: Binding<MullvadAlert?>) -> some View {
        modifier(AlertModifier(alert: item))
    }

    func mullvadInputAlert(item: Binding<MullvadInputAlert?>) -> some View {
        modifier(InputAlertModifier(alert: item))
    }
}

#Preview {
    Text("Hello, World!")
        .mullvadAlert(
            item:
                .constant(
                    .init(
                        type: .warning,
                        messages: ["Something needs to be done"],
                        actions: [
                            .init(
                                type: .danger,
                                title: "Do it!",
                                handler: {}
                            ),
                            .init(
                                type: .default,
                                title: "Cancel",
                                handler: {}
                            ),
                        ]
                    )
                )
        )
}

#Preview("Input") {
    Text("Hello, World!")
        .mullvadInputAlert(
            item:
                .constant(
                    .init(
                        title: "Title",
                        placeholder: "Placeholder",
                        action: .init(
                            type: .default,
                            title: "Do it!",
                            identifier: nil,
                            handler: { _ in }
                        ),
                        validate: nil,
                        dismissButtonTitle: "Cancel"
                    )
                )
        )
}
