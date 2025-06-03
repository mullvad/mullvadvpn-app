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
                VStack {
                    Spacer()
                    VStack(spacing: 16) {
                        switch alert.type {
                        case .error, .warning:
                            Image.mullvadIconAlert
                                .resizable()
                                .frame(width: 48, height: 48)
                        }
                        Text(alert.message)
                            .font(.mullvadSmall)
                            .foregroundColor(.mullvadTextPrimary.opacity(0.6))
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

struct ClearBackgroundView: UIViewRepresentable {
    func makeUIView(context: Context) -> UIView {
        return InnerView()
    }

    func updateUIView(_ uiView: UIView, context: Context) {}

    private class InnerView: UIView {
        override func didMoveToWindow() {
            super.didMoveToWindow()

            superview?.superview?.backgroundColor = .init(red: 0, green: 0, blue: 0, alpha: 0.5)
        }
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
