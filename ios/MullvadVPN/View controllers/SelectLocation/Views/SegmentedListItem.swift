import SwiftUI

struct SegmentedListItem<Label: View, Segment: View, GroupedContent: View>: View {
    @State private var segmentHeight: CGFloat = UIMetrics.LocationList.cellMinHeight

    var level: Int = 0
    var isLastInList: Bool = true
    var isDisabled: Bool = false
    var accessibilityIdentifier: AccessibilityIdentifier?
    var accessibilityLabel: String = ""
    @ViewBuilder let label: () -> Label
    @ViewBuilder var segment: () -> Segment?
    @ViewBuilder var groupedContent: () -> GroupedContent?
    let onSelect: () -> Void
    var onSecondarySelect: (() -> Void)?

    private var topRadius: CGFloat {
        level == 0 ? UIMetrics.LocationList.cellCornerRadius : 0
    }
    private var bottomRadius: CGFloat {
        isLastInList && (groupedContent() == nil || groupedContent() is EmptyView)
            ? UIMetrics.LocationList.cellCornerRadius
            : 0
    }

    var body: some View {
        HStack(spacing: 2) {
            Button {
                withAnimation(.easeInOut(duration: 0.15)) {
                    onSelect()
                }
            } label: {
                label()
                    .background(Color.colorForLevel(level))
                    .sizeOfView {
                        segmentHeight = $0.height
                    }
            }
            .disabled(isDisabled)

            segment()
                .frame(width: UIMetrics.LocationList.cellMinHeight, height: segmentHeight)
                .background(Color.colorForLevel(level))
                .contentShape(Rectangle())
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel(accessibilityLabel)
        .accessibilityIdentifier(accessibilityIdentifier)
        .accessibilityAction(named: Text("Select \(accessibilityLabel)")) {
            withAnimation(.easeInOut(duration: 0.15)) {
                onSelect()
            }
        }
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

        groupedContent()
    }
}

private extension Color {
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
