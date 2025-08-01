import SwiftUI

struct MullvadAlert: Identifiable {
    enum AlertType {
        case warning
        case error
    }

    enum ActionType {
        case danger
        case normal
    }

    struct Action {
        let type: MainButtonStyle.Style
        let title: String
        let identifier: AccessibilityIdentifier?
        let handler: () async -> Void
    }

    let id = UUID()
    let type: AlertType
    let message: String
    let action: Action?
    let dismissButtonTitle: String
}

struct AlertModifier: ViewModifier {
    @Binding var alert: MullvadAlert?
    @State var loading = false
    func body(content: Content) -> some View {
        content
            .fullScreenCover(item: $alert) { alert in
                VStack {
                    Spacer()
                    VStack(spacing: 16) {
                        switch alert.type {
                        case .error, .warning:
                            Image.mullvadIconAlert
                                .resizable()
                                .frame(width: 48, height: 48)
                        }
                        HStack {
                            if let attributed = try? AttributedString(
                                markdown: alert.message,
                                options: .init(
                                    interpretedSyntax: .inlineOnlyPreservingWhitespace,
                                    languageCode: ApplicationLanguage.currentLanguage.id
                                )
                            ) {
                                Text(attributed)
                                    .font(.mullvadSmall)
                                    .foregroundColor(.mullvadTextPrimary.opacity(0.6))
                            } else {
                                Text(alert.message)
                                    .font(.mullvadSmall)
                                    .foregroundColor(.mullvadTextPrimary.opacity(0.6))
                            }
                            Spacer()
                        }
                        VStack(spacing: 16) {
                            if let action = alert.action {
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
}

#Preview {
    Text("Hello, World!")
        .mullvadAlert(
            item:
            .constant(
                .init(
                    type: .warning,
                    message: "Something needs to be done",
                    action: .init(
                        type: .danger,
                        title: "Do it!",
                        identifier: nil,
                        handler: {}
                    ),
                    dismissButtonTitle: NSLocalizedString(
                        "CANCEL_TITLE_BUTTON",
                        tableName: "Common",
                        value: "Cancel",
                        comment: ""
                    )
                )
            )
        )
}
