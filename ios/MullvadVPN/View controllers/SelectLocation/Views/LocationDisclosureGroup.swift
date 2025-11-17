import SwiftUI

struct LocationDisclosureGroup<Label: View, Content: View, ContextMenu: View>: View {
    @Binding private var isExpanded: Bool

    let level: Int
    let isLastInList: Bool
    let isActive: Bool
    let label: () -> Label
    let content: () -> Content
    let onSelect: (() -> Void)?
    let contextMenu: () -> ContextMenu
    let accessibilityIdentifier: AccessibilityIdentifier?

    private var topRadius: CGFloat {
        level == 0 ? 16 : 0
    }
    private var bottomRadius: CGFloat {
        isLastInList && !isExpanded ? 16 : 0
    }

    init(
        level: Int,
        isLastInList: Bool,
        isActive: Bool = true,
        isExpanded: Binding<Bool>,
        @ViewBuilder contextMenu: @escaping () -> ContextMenu,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        @ViewBuilder content: @escaping () -> Content,
        @ViewBuilder label: @escaping () -> Label,
        onSelect: (() -> Void)? = nil,
    ) {
        self.level = level
        self.isLastInList = isLastInList
        self.isActive = isActive
        self._isExpanded = isExpanded
        self.accessibilityIdentifier = accessibilityIdentifier

        self.label = label
        self.content = content
        self.onSelect = onSelect
        self.contextMenu = contextMenu
    }

    var body: some View {
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
        .clipShape(
            UnevenRoundedRectangle(
                cornerRadii: .init(
                    topLeading: topRadius,
                    bottomLeading: bottomRadius,
                    bottomTrailing: bottomRadius,
                    topTrailing: topRadius
                )
            )
        )
        .padding(.top, level == 0 ? 4 : 1)
        .contextMenu {
            contextMenu()
        }

        if isExpanded {
            content()
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
