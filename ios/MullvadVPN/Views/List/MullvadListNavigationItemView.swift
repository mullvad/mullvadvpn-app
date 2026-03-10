import SwiftUI

struct MullvadListNavigationItem: Hashable, Identifiable {
    enum State: CustomStringConvertible {
        case off
        case inUse
        case warning(String)
        case error(String)

        var description: String {
            switch self {
            case .off:
                NSLocalizedString("Off", comment: "")
            case .inUse:
                NSLocalizedString("In use", comment: "")
            case .warning(let message), .error(let message):
                NSLocalizedString(message, comment: "")
            }
        }
    }

    let id: UUID
    let title: String
    let state: State?
    let detail: String?
    let accessibilityIdentifier: AccessibilityIdentifier?
    let didSelect: (() -> Void)?

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: MullvadListNavigationItem, rhs: MullvadListNavigationItem) -> Bool {
        lhs.id == rhs.id
    }
}

struct MullvadListNavigationItemView: View {
    private let title: String
    private let state: MullvadListNavigationItem.State?
    private let detail: String?
    private let accessibilityIdentifier: AccessibilityIdentifier?
    private let didSelect: (() -> Void)?
    @State private var isPressed = false
    init(
        item: MullvadListNavigationItem
    ) {
        self.title = item.title
        self.state = item.state
        self.detail = item.detail.flatMap { $0.isEmpty ? nil : $0 }
        self.accessibilityIdentifier = item.accessibilityIdentifier
        self.didSelect = item.didSelect
    }

    var body: some View {
        Button {
            didSelect?()
        } label: {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(title)
                        .foregroundStyle(Color(.Cell.titleTextColor))
                        .font(.mullvadSmallSemiBold)
                        .fixedSize(horizontal: false, vertical: true)
                    if let detail {
                        Text(detail)
                            .foregroundStyle(Color(.Cell.detailTextColor.withAlphaComponent(0.6)))
                            .font(.mullvadMiniSemiBold)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                }
                Spacer()
                if let state {
                    HStack(alignment: .center) {
                        Text(state.description)
                            .foregroundStyle(Color(.Cell.titleTextColor.withAlphaComponent(0.6)))
                            .font(.mullvadTiny)
                            .fixedSize(horizontal: false, vertical: true)
                        if case .warning = state {
                            Image.mullvadIconStateIssue
                                .resizable()
                                .frame(width: 18, height: 18)
                        } else if case .error = state {
                            Image.mullvadIconStateOffline
                                .resizable()
                                .frame(width: 18, height: 18)
                        }
                    }
                }
                Image(.iconChevron)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 11)
            .background(
                isPressed
                    ? Color.MullvadButton.primaryPressed
                    : Color.MullvadButton
                        .primary
            )
        }
        .accessibilityIdentifier(accessibilityIdentifier?.asString ?? title)
        .onButtonPressedChange { isPressed in
            self.isPressed = isPressed
        }
    }
}

fileprivate extension View {
    func onButtonPressedChange(_ onChange: @escaping (Bool) -> Void) -> some View {
        buttonStyle(
            MullvadListButtonStyle(onButtonPressedChange: onChange)
        )
    }
}

private struct MullvadListButtonStyle: ButtonStyle {
    let onButtonPressedChange: (Bool) -> Void
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .onChange(
                of: configuration.isPressed,
                {
                    onButtonPressedChange(configuration.isPressed)
                })
    }
}

#Preview {
    Text("")
        .sheet(isPresented: .constant(true)) {
            MullvadList(
                [
                    MullvadListNavigationItem(
                        id: UUID(),
                        title: "Test method",
                        state: .inUse,
                        detail: "Very good method",
                        accessibilityIdentifier: nil,
                        didSelect: { print("selected") }
                    ),
                    MullvadListNavigationItem(
                        id: UUID(),
                        title: "Test method2",
                        state: .error("Unsupported cipher"),
                        detail: "Shadowsocks",
                        accessibilityIdentifier: nil,
                        didSelect: { print("selected") }
                    ),
                ],
                header: { EmptyView() },
                footer: { EmptyView() },
                content: { item in
                    MullvadListNavigationItemView(item: item)
                }
            )
        }
}
