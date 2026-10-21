import SwiftUI

struct MullvadAlert: Identifiable, Equatable {
    enum AlertType {
        case none
        case info
        case warning
        case error
    }

    enum DismissButtonPosition {
        case top
        case bottom
    }

    struct Action: Identifiable {
        let id = UUID()
        let type: MullvadButton.Style
        let title: LocalizedStringKey
        let identifier: AccessibilityIdentifier?
        let handler: () async -> Void

        init(
            type: MullvadButton.Style,
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
    let title: LocalizedStringKey?
    let messages: [LocalizedStringKey]
    let customView: AnyView?
    let actions: [Action]

    init(
        type: AlertType,
        title: LocalizedStringKey? = nil,
        messages: [LocalizedStringKey] = [],
        customView: AnyView? = nil,
        actions: [Action] = []
    ) {
        self.type = type
        self.title = title
        self.messages = messages
        self.customView = customView
        self.actions = actions
    }

    static func == (lhs: MullvadAlert, rhs: MullvadAlert) -> Bool {
        lhs.id == rhs.id
    }
}

struct MullvadInputAlert: Identifiable, Equatable {
    struct Action {
        let type: MullvadButton.Style
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

    static func == (lhs: MullvadInputAlert, rhs: MullvadInputAlert) -> Bool {
        lhs.id == rhs.id
    }
}

struct AlertModifier: ViewModifier {
    @Binding var alert: MullvadAlert?
    @State var loading = false
    @State private var internalAlert: MullvadAlert?
    @State private var scrollViewHeight: CGFloat = 0

    func showWithoutAnimation() {
        var transaction = Transaction()
        transaction.disablesAnimations = true
        withTransaction(transaction) {
            internalAlert = alert
        }
    }

    func body(content: Content) -> some View {
        content
            .fullScreenCover(item: $internalAlert) { alert in
                alertView(for: alert)
            }
            .onChange(of: alert) {
                showWithoutAnimation()
            }
            .onAppear {
                showWithoutAnimation()
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
            alertTitle(for: alert.title)
            alertMessage(alert.messages, customView: alert.customView)
            VStack(spacing: 16) {
                ForEach(alert.actions) { action in
                    alertAction(for: action)
                }
            }
        }
        .padding(.horizontal, 16)
        .padding(.bottom, 16)
        .padding(.top, 24)
        .background(Color.mullvadBackground)
        .cornerRadius(8)
    }

    @ViewBuilder
    private func alertIcon(for type: MullvadAlert.AlertType) -> some View {
        switch type {
        case .none:
            EmptyView()
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
    private func alertTitle(for title: LocalizedStringKey?) -> some View {
        if let title {
            HStack {
                Text(title)
                    .font(.mullvadSmallSemiBold)
                    .foregroundStyle(Color.mullvadTextPrimary)
                Spacer()
            }
        }
    }

    @ViewBuilder
    private func alertMessage(_ messages: [LocalizedStringKey], customView: AnyView?) -> some View {
        ScrollView {
            VStack {
                ForEach(Array(messages.enumerated()), id: \.offset) { _, text in
                    HStack {
                        Text(text)
                            .font(.mullvadSmall)
                            .foregroundColor(.mullvadTextSecondary)
                        Spacer()
                    }
                }
                customView
            }
            .sizeOfView { size in
                scrollViewHeight = size.height
            }
        }
        .contentMargins(0, for: .scrollContent)
        .frame(maxHeight: scrollViewHeight)
    }

    @ViewBuilder
    private func alertAction(for action: MullvadAlert.Action) -> some View {
        MullvadButton(
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
    @State private var internalAlert: MullvadInputAlert?

    func showWithoutAnimation() {
        var transaction = Transaction()
        transaction.disablesAnimations = true
        withTransaction(transaction) {
            internalAlert = alert
        }
    }

    func body(content: Content) -> some View {
        content
            .fullScreenCover(item: $internalAlert) { alert in
                InputAlertContent(alert: alert) {
                    self.alert = nil
                }
            }
            .onChange(of: alert) {
                showWithoutAnimation()
            }
            .onAppear {
                showWithoutAnimation()
            }
    }
}

private struct InputAlertContent: View {
    let alert: MullvadInputAlert
    let dismiss: () -> Void

    @State private var text = ""
    @State private var loading = false
    @FocusState private var isFocused: Bool

    var body: some View {
        VStack {
            Spacer()
            VStack(alignment: .leading, spacing: 16) {
                Text(alert.title)
                    .font(.mullvadLarge)
                    .foregroundStyle(Color.mullvadTextPrimary)
                    .lineLimit(nil)
                    .fixedSize(horizontal: false, vertical: true)

                ConfigurableTextField(
                    placeholder: alert.placeholder,
                    text: $text,
                    isFocused: $isFocused,
                    borderStyle: .constant(.normal)
                )
                .onAppear {
                    isFocused = true
                }

                VStack(spacing: 16) {
                    MullvadButton(
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
                    MullvadButton(
                        text: alert.dismissButtonTitle,
                        style: .secondary,
                        action: { self.dismiss() }
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
}

struct SpinnerModifier: ViewModifier {
    let isPresented: Bool
    @State private var isPresentedInternal: Bool
    init(isPresented: Bool) {
        self.isPresented = isPresented
        self.isPresentedInternal = isPresented
    }
    func body(content: Content) -> some View {
        content
            .fullScreenCover(isPresented: $isPresentedInternal) {
                VStack {
                    Spacer()
                    HStack {
                        Spacer()
                        ProgressView()
                            .progressViewStyle(MullvadProgressViewStyle())
                        Spacer()
                    }
                    Spacer()
                }
                .background(ClearBackgroundView())
            }
            .onChange(of: isPresented) {
                var transaction = Transaction()
                transaction.disablesAnimations = true
                withTransaction(transaction) {
                    isPresentedInternal = isPresented
                }
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

    func mullvadLoadingSpinner(isPresented: Bool) -> some View {
        modifier(
            SpinnerModifier(isPresented: isPresented)
        )
    }
}

#Preview {
    Text("Hello, World!")
        .mullvadAlert(
            item:
                .constant(
                    .init(
                        type: .warning,
                        title: "Title",
                        messages: ["Something needs to be done"],
                        actions: [
                            .init(
                                type: .destructivePrimary,
                                title: "Do it!",
                                handler: {}
                            ),
                            .init(
                                type: .secondary,
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
                            type: .primary,
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

#Preview {
    Text("Hello, World!")
        .mullvadLoadingSpinner(isPresented: true)
}
