import SwiftUI

struct MullvadListActionItem: Hashable, Identifiable {
    var id: String
    let title: LocalizedStringKey
    let state: LocalizedStringKey?
    let detail: LocalizedStringKey?
    let accessibilityIdentifier: AccessibilityIdentifier?
    let pressed: (() -> Void)?

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: MullvadListActionItem, rhs: MullvadListActionItem) -> Bool {
        lhs.id == rhs.id
    }
}

struct MullvadListActionItemView<Icon: View>: View {
    private let title: LocalizedStringKey
    private let state: LocalizedStringKey?
    private let detail: LocalizedStringKey?
    private let icon: Icon?
    private let accessibilityIdentifier: AccessibilityIdentifier?
    private let pressed: (() -> Void)?

    init(
        item: MullvadListActionItem,
        @ViewBuilder icon: () -> Icon?
    ) {
        self.title = item.title
        self.state = item.state
        self.detail = item.detail
        self.accessibilityIdentifier = item.accessibilityIdentifier
        self.pressed = item.pressed
        self.icon = icon()
    }

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .foregroundStyle(Color(.Cell.titleTextColor))
                    .font(.mullvadSmallSemiBold)
                if let detail {
                    Text(detail)
                        .foregroundStyle(Color(.Cell.detailTextColor.withAlphaComponent(0.6)))
                        .font(.mullvadMiniSemiBold)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
            Spacer()
            if let state {
                Text(state)
                    .foregroundStyle(Color(.Cell.titleTextColor.withAlphaComponent(0.6)))
                    .font(.mullvadTiny)
            }
            if let icon {
                Button {
                    pressed?()
                } label: {
                    icon
                }
                .accessibilityIdentifier(accessibilityIdentifier)
                .padding(.leading, 8)
            }
        }
        .padding(
            EdgeInsets(
                top: 8,
                leading: UIMetrics.contentLayoutMargins.leading,
                bottom: 8,
                trailing: UIMetrics.contentLayoutMargins.trailing
            )
        )
        .background(Color.MullvadList.background)
    }
}

#Preview {
    Text("")
        .sheet(isPresented: .constant(true)) {
            MullvadList(
                [
                    MullvadListActionItem(
                        id: "1",
                        title: "Blind mole",
                        state: nil,
                        detail: "Created: 2024-05-08",
                        accessibilityIdentifier: nil,
                        pressed: {
                            print("selected")
                        }
                    ),
                    MullvadListActionItem(
                        id: "2",
                        title: "Tall mole",
                        state: "Current Device",
                        detail: "Created: 2024-05-08",
                        accessibilityIdentifier: nil,
                        pressed: nil
                    ),
                ],
                header: { EmptyView() },
                footer: { EmptyView() },
                content: { item in
                    MullvadListActionItemView(item: item) {
                        if item.pressed != nil {
                            Image.mullvadIconClose
                        }
                    }
                }
            )
        }
}
