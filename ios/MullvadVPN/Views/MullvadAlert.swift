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
        let title: LocalizedStringKey
        let identifier: AccessibilityIdentifier?
        let handler: () async -> Void
    }

    let id = UUID()
    let type: AlertType
    let message: LocalizedStringKey
    let action: Action?
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
            alertMessage(alert.message)
            VStack(spacing: 16) {
                alertAction(for: alert.action)
                alertAction(for: MullvadAlert.Action(
                    type: .default,
                    title: alert.dismissButtonTitle,
                    identifier: nil,
                    handler: { self.alert = nil }
                ))
            }
        }
        .padding()
        .background(Color.mullvadBackground)
        .cornerRadius(8)
    }

    @ViewBuilder
    private func alertIcon(for type: MullvadAlert.AlertType) -> some View {
        switch type {
        case .error, .warning:
            Image.mullvadIconAlert
                .resizable()
                .frame(width: 48, height: 48)
        }
    }

    @ViewBuilder
    private func alertMessage(_ message: LocalizedStringKey) -> some View {
        HStack {
            Text(message)
                .font(.mullvadSmall)
                .foregroundColor(.mullvadTextPrimary.opacity(0.6))
            Spacer()
        }
    }

    @ViewBuilder
    private func alertAction(for action: MullvadAlert.Action?) -> some View {
        if let action = action {
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
        } else {
            EmptyView()
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
                    dismissButtonTitle: "Cancel"
                )
            )
        )
}
