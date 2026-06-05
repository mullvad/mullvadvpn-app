import SwiftUI

struct SegmentedListItem<Leading: View, Trailing: View, Segment: View, GroupedContent: View>: View {
    @Environment(\.isNestedInSegmentedListItem) private var isNestedInSegmentedListItem

    @State private var segmentHeight: CGFloat = UIMetrics.LocationList.cellMinHeight

    var level: Int = 0
    var isLastInList: Bool = true
    var isDisabled: Bool = false
    var accessibilityIdentifier: AccessibilityIdentifier?
    var accessibilityLabel: String = ""
    /// A `Leading` sub component. Intended to be used for leading elements, such as titles, status indicators etc.
    @ViewBuilder var leading: () -> Leading?
    /// A `Trailing` sub component. Intended to be used for trailing elements, such as subtitles, buttons etc.
    @ViewBuilder var trailing: () -> Trailing?
    /// A `Segment` sub component. Splits the list item in two, with a trailing square typically used for buttons to expand a list item.
    @ViewBuilder var segment: () -> Segment?
    /// A `GroupedContent` sub component. Adds sub items to the list. Typically used in multi-choice settings or expanded lists.
    @ViewBuilder var groupedContent: () -> GroupedContent?
    var footer: MullvadInfoView? = nil
    var onSelect: (() -> Void)? = nil

    private var topRadius: CGFloat {
        (level == 0 && !isNestedInSegmentedListItem) ? UIMetrics.LocationList.cellCornerRadius : 0
    }
    private var bottomRadius: CGFloat {
        let groupedContent = groupedContent()

        return isLastInList && (groupedContent == nil || groupedContent is EmptyView)
            ? UIMetrics.LocationList.cellCornerRadius
            : 0
    }

    var body: some View {
        HStack(spacing: 2) {
            Button {
                withAnimation(.easeInOut(duration: 0.15)) {
                    onSelect?()
                }
            } label: {
                let trailing = trailing()

                HStack(spacing: 8) {
                    leading()
                    Spacer()
                    trailing
                }
                .padding(.leading, 16)
                .padding(.trailing, trailing == nil ? 16 : 0)
                .background(Color.colorForIndentationLevel(level))
                .sizeOfView {
                    segmentHeight = $0.height
                }
            }
            .disabled(isDisabled)

            segment()?
                .frame(width: UIMetrics.LocationList.cellMinHeight, height: segmentHeight)
                .contentShape(Rectangle())
                .background(Color.colorForIndentationLevel(level))
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel(accessibilityLabel)
        .accessibilityIdentifier(accessibilityIdentifier)
        .accessibilityAction(named: Text("Select \(accessibilityLabel)")) {
            withAnimation(.easeInOut(duration: 0.15)) {
                onSelect?()
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

        groupedContent()
            .environment(\.isNestedInSegmentedListItem, true)
            .padding(.top, 1)

        footer
            .padding(.horizontal, 16)
            .padding(.top, 2)
    }
}

#Preview {
    @Previewable @State var inputText: String = ""
    @Previewable @State var toggleState: Bool = false

    let itemFactory = SegmentedListItemFactory()

    VStack(spacing: 0) {
        SegmentedListItem(
            leading: {
                itemFactory.leading(for: .generic(title: "Setting - custom"))
            },
            trailing: {
                itemFactory.trailing(
                    for: .custom(items: [
                        .breadcrumb(.warning(.root)),
                        .string("Custom text"),
                        .button(icon: .info, onSelect: { print("onSelect") }),
                        .button(icon: .close, onSelect: { print("onClose") }),
                    ])
                )
            },
            footer: MullvadInfoView(
                bodyText: "Some text. ",
                link: "A link",
                onTapLink: { print("onLinkTap") }
            )
        )

        Spacer()
    }
    .background(Color.mullvadBackground)
}

// MARK: - Nesting

private struct IsNestedInSegmentedListItemKey: EnvironmentKey {
    static let defaultValue = false
}

extension EnvironmentValues {
    var isNestedInSegmentedListItem: Bool {
        get { self[IsNestedInSegmentedListItemKey.self] }
        set { self[IsNestedInSegmentedListItemKey.self] = newValue }
    }
}

// MARK: - Color helpers

private extension Color {
    static func colorForIndentationLevel(_ level: Int) -> Color {
        switch level {
        case 1: Color.MullvadList.Item.level1
        case 2: Color.MullvadList.Item.level2
        case 3: Color.MullvadList.Item.level3
        case 4: Color.MullvadList.Item.level4
        default: Color.MullvadList.Item.parent
        }
    }
}
