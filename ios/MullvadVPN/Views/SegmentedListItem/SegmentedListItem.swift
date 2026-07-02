import SwiftUI

struct SegmentedListItem<Leading: View>: View {
    enum UserInteraction {
        case enabled
        case enabledWithoutHighlight
        case disabled
    }

    @Environment(\.isNestedInSegmentedListItem) private var isNestedInSegmentedListItem
    @State private var segmentHeight: CGFloat = UIMetrics.LocationList.cellMinHeight

    var level: Int
    var isLastInList: Bool
    var userInteraction: UserInteraction
    var accessibilityIdentifier: AccessibilityIdentifier?
    var accessibilityLabel: String
    /// A leading sub component. Intended to be used for leading elements, such as titles, status indicators etc.
    @ViewBuilder var leading: () -> Leading
    /// A trailing sub component. Intended to be used for trailing elements, such as subtitles, buttons etc.
    let trailing: AnyView?
    /// A segment sub component. Splits the list item in two, with a trailing square typically used for buttons to expand a list item.
    let segment: AnyView?
    /// A grouped content sub component. Adds sub items to the list. Typically used in multi-choice settings or expanded lists.
    let groupedContent: AnyView?
    var footer: MullvadInfoView?
    var onSelect: (() -> Void)?

    /// The optional `trailing`, `segment` and `groupedContent` view builders default to `EmptyView`, so callers
    /// only specify the slots they need. An omitted slot is detected via its `EmptyView` type and stored as `nil`
    /// rather than an empty `AnyView`.
    init<Trailing: View, Segment: View, GroupedContent: View>(
        level: Int = 0,
        isLastInList: Bool = true,
        userInteraction = .enabled,
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        accessibilityLabel: String = "",
        @ViewBuilder leading: @escaping () -> Leading,
        @ViewBuilder trailing: () -> Trailing = { EmptyView() },
        @ViewBuilder segment: () -> Segment = { EmptyView() },
        @ViewBuilder groupedContent: () -> GroupedContent = { EmptyView() },
        footer: MullvadInfoView? = nil,
        onSelect: (() -> Void)? = nil
    ) {
        self.level = level
        self.isLastInList = isLastInList
        self.userInteraction = userInteraction
        self.accessibilityIdentifier = accessibilityIdentifier
        self.accessibilityLabel = accessibilityLabel
        self.leading = leading
        self.trailing = trailing().typeErase()
        self.segment = segment().typeErase()
        self.groupedContent = groupedContent().typeErase()
        self.footer = footer
        self.onSelect = onSelect
    }

    private var topRadius: CGFloat {
        (level == 0 && !isNestedInSegmentedListItem) ? UIMetrics.LocationList.cellCornerRadius : 0
    }
    private var bottomRadius: CGFloat {
        isLastInList && groupedContent == nil
            ? UIMetrics.LocationList.cellCornerRadius
            : 0
    }

    var body: some View {
        HStack(spacing: 2) {
            let button = Button {
                withAnimation(.easeInOut(duration: 0.15)) {
                    onSelect?()
                }
            } label: {
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

            switch userInteraction {
            case .enabled:
                button
                    .buttonStyle(PlainButtonStyle())
                    .disabled(false)
            case .enabledWithoutHighlight:
                button
                    .buttonStyle(StaticButtonStyle())
                    .disabled(false)
            case .disabled:
                button
                    .buttonStyle(PlainButtonStyle())
                    .disabled(true)
            }

            segment
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

        groupedContent
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
