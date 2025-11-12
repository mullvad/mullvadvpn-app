import SwiftUI

struct LocationDisclosureGroup<Label: View, Content: View>: View {
    @Binding private var isExpanded: Bool

    let level: Int
    let isActive: Bool
    let label: () -> Label
    let content: () -> Content
    let onSelect: (() -> Void)?
    let accessibilityIdentifier: AccessibilityIdentifier?

    init(
        level: Int,
        isActive: Bool = true,
        isExpanded: Binding<Bool>,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        @ViewBuilder content: @escaping () -> Content,
        @ViewBuilder label: @escaping () -> Label,
        onSelect: (() -> Void)? = nil,
    ) {
        self.level = level
        self.isActive = isActive
        self._isExpanded = isExpanded
        self.accessibilityIdentifier = accessibilityIdentifier

        self.label = label
        self.content = content
        self.onSelect = onSelect
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 2) {
                Button {
                    onSelect?()
                } label: {
                    HStack {
                        label()
                        Spacer()
                    }
                    .frame(maxHeight: .infinity)
                    .background {
                        Color.colorForLevel(level)
                    }
                }
                .disabled(!isActive)
                Button {
                    withAnimation {
                        isExpanded.toggle()
                    }
                } label: {
                    Image.mullvadIconChevron
                        .rotationEffect(.degrees(isExpanded ? -90 : 90))
                        .padding(16)
                        .frame(maxHeight: .infinity)
                        .background {
                            Color.colorForLevel(level)
                        }
                }
                .accessibilityLabel(isExpanded ? Text("Collapse") : Text("Expand"))
                .accessibilityIdentifier(.expandButton)
                .contentShape(Rectangle())
            }
            .accessibilityElement(children: .combine)
            .accessibilityIdentifier(accessibilityIdentifier)

            if isExpanded {
                LazyVStack(spacing: 1) {
                    content()
                }
                .padding(.top, 1)
            }
        }
    }
}

extension Color {
    static func colorForLevel(_ level: Int) -> Color {
        switch level {
        case 1: Color.MullvadList.Item.child1
        case 2: Color.MullvadList.Item.child2
        case 3: Color.MullvadList.Item.child3
        case 4: Color.MullvadList.Item.child4
        default: Color.MullvadList.Item.parent
        }
    }
}
